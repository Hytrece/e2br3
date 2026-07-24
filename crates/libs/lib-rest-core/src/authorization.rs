use crate::{Error, Result};
use lib_core::authorization::{
	authorize_contextual_mutation, authorize_contextual_read, authorize_subject,
	policy_registry, AuthorizationContext, AuthorizationDenial, AuthorizedMutation,
	AuthorizedRead, AuthorizedSubject, CaseChildResource, CaseCreateProposal,
	CaseResource, Collection, Existing, Parent, PolicySnapshotVersion, Proposed,
	RequestAuthorizationSnapshot,
};
use lib_core::ctx::{Ctx, ROLE_SYSTEM_ADMIN};
use lib_core::model::authorization::{
	AuthorizationFactLoadError, AuthorizationFactLoader, CaseMutationKind,
};
use lib_core::model::store::set_full_context_from_ctx_dbx;
use lib_core::model::ModelManager;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

pub fn denied(denial: AuthorizationDenial) -> Error {
	Error::PermissionDenied {
		required_permission: format!(
			"{} ({:?})",
			denial.action_id(),
			denial.reason()
		),
	}
}

trait RequestBoundPermit {
	fn principal_id(&self) -> Uuid;
	fn organization_id(&self) -> Uuid;
	fn snapshot_version(&self) -> &PolicySnapshotVersion;
}

trait PermitEvidence: RequestBoundPermit {
	fn target_organization_id(&self) -> Option<Uuid>;
}

impl RequestBoundPermit for AuthorizedSubject {
	fn principal_id(&self) -> Uuid {
		self.principal_id()
	}
	fn organization_id(&self) -> Uuid {
		self.organization_id()
	}
	fn snapshot_version(&self) -> &PolicySnapshotVersion {
		self.snapshot_version()
	}
}

impl<C: AuthorizationContext> RequestBoundPermit for AuthorizedRead<'_, C> {
	fn principal_id(&self) -> Uuid {
		self.principal_id()
	}
	fn organization_id(&self) -> Uuid {
		self.organization_id()
	}
	fn snapshot_version(&self) -> &PolicySnapshotVersion {
		self.snapshot_version()
	}
}

impl<C: AuthorizationContext> PermitEvidence for AuthorizedRead<'_, C> {
	fn target_organization_id(&self) -> Option<Uuid> {
		self.target_organization_id()
	}
}

impl<C: AuthorizationContext> RequestBoundPermit for AuthorizedMutation<'_, C> {
	fn principal_id(&self) -> Uuid {
		self.principal_id()
	}
	fn organization_id(&self) -> Uuid {
		self.organization_id()
	}
	fn snapshot_version(&self) -> &PolicySnapshotVersion {
		self.snapshot_version()
	}
}

impl<C: AuthorizationContext> PermitEvidence for AuthorizedMutation<'_, C> {
	fn target_organization_id(&self) -> Option<Uuid> {
		self.target_organization_id()
	}
}

pub fn rls_ctx_for_authorized_subject(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	permit: &AuthorizedSubject,
) -> Result<Ctx> {
	validate_request_binding(request_ctx, snapshot, permit)?;
	Ok(request_ctx.clone())
}

pub async fn with_authorized_subject_action<T, F>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	mm: &ModelManager,
	action_id: &'static str,
	operation: F,
) -> Result<T>
where
	F: for<'ctx> FnOnce(
		&'ctx Ctx,
		&'ctx ModelManager,
	) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'ctx>>,
{
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(lib_core::model::Error::from)?;
	if let Err(error) = set_full_context_from_ctx_dbx(dbx, request_ctx).await {
		let _ = dbx.rollback_txn().await;
		return Err(error.into());
	}
	let result = async {
		let action =
			policy_registry().subject_action(action_id).ok_or_else(|| {
				Error::AccessDenied {
					required_role: format!("registered {action_id} action"),
				}
			})?;
		let permit = authorize_subject(action, snapshot).map_err(denied)?;
		let authorized_ctx =
			rls_ctx_for_authorized_subject(request_ctx, snapshot, &permit)?;
		operation(&authorized_ctx, mm).await
	}
	.await;
	finish_fact_transaction(dbx, result).await
}

pub async fn with_authorized_case_collection<T, F>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	mm: &ModelManager,
	operation: F,
) -> Result<T>
where
	F: for<'ctx> FnOnce(
		&'ctx Ctx,
		&'ctx ModelManager,
	) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'ctx>>,
{
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(lib_core::model::Error::from)?;
	if let Err(error) = set_full_context_from_ctx_dbx(dbx, request_ctx).await {
		let _ = dbx.rollback_txn().await;
		return Err(error.into());
	}
	let result = async {
		let context = AuthorizationFactLoader::new(dbx, snapshot).case_collection();
		let action = policy_registry()
			.context_action::<Collection<CaseResource>>("case.list")
			.ok_or_else(|| Error::AccessDenied {
				required_role: "registered case.list action".to_string(),
			})?;
		let permit =
			authorize_contextual_read(action, snapshot, context).map_err(denied)?;
		let authorized_ctx =
			rls_ctx_for_authorized_read(request_ctx, snapshot, &permit)?;
		operation(&authorized_ctx, mm).await
	}
	.await;
	finish_fact_transaction(dbx, result).await
}

pub async fn with_authorized_case_read<T, F>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	mm: &ModelManager,
	case_id: Uuid,
	action_id: &'static str,
	operation: F,
) -> Result<T>
where
	F: for<'ctx> FnOnce(
		&'ctx Ctx,
		&'ctx ModelManager,
	) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'ctx>>,
{
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(lib_core::model::Error::from)?;
	if let Err(error) = set_full_context_from_ctx_dbx(dbx, request_ctx).await {
		let _ = dbx.rollback_txn().await;
		return Err(error.into());
	}
	let result = async {
		let context = AuthorizationFactLoader::new(dbx, snapshot)
			.case_existing(case_id)
			.await
			.map_err(map_fact_load_error)?;
		let action = policy_registry()
			.context_action::<Existing<CaseResource>>(action_id)
			.ok_or_else(|| Error::AccessDenied {
				required_role: format!("registered {action_id} action"),
			})?;
		let permit =
			authorize_contextual_read(action, snapshot, context).map_err(denied)?;
		let authorized_ctx =
			rls_ctx_for_authorized_read(request_ctx, snapshot, &permit)?;
		operation(&authorized_ctx, mm).await
	}
	.await;
	finish_fact_transaction(dbx, result).await
}

pub async fn with_authorized_case_create<T, F>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	mm: &ModelManager,
	operation: F,
) -> Result<T>
where
	F: for<'ctx> FnOnce(
		&'ctx Ctx,
		&'ctx ModelManager,
	) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'ctx>>,
{
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(lib_core::model::Error::from)?;
	if let Err(error) = set_full_context_from_ctx_dbx(dbx, request_ctx).await {
		let _ = dbx.rollback_txn().await;
		return Err(error.into());
	}
	let loader = AuthorizationFactLoader::new(dbx, snapshot);
	if let Err(error) = loader.lock_and_verify_revisions().await {
		let _ = dbx.rollback_txn().await;
		return Err(map_fact_load_error(error));
	}
	let result = async {
		let context =
			loader.case_create_for_verified_mutation(request_ctx.organization_id());
		let action = policy_registry()
			.context_action::<Proposed<CaseCreateProposal>>("case.create")
			.ok_or_else(|| Error::AccessDenied {
				required_role: "registered case.create action".to_string(),
			})?;
		let permit = authorize_contextual_mutation(action, snapshot, context)
			.map_err(denied)?;
		let authorized_ctx =
			rls_ctx_for_authorized_mutation(request_ctx, snapshot, &permit)?;
		operation(&authorized_ctx, mm).await
	}
	.await;
	finish_fact_transaction(dbx, result).await
}

pub async fn with_authorized_case_mutation<T, F>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	mm: &ModelManager,
	case_id: Uuid,
	action_id: &'static str,
	mutation_kind: CaseMutationKind,
	operation: F,
) -> Result<T>
where
	F: for<'ctx> FnOnce(
		&'ctx Ctx,
		&'ctx ModelManager,
	) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'ctx>>,
{
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(lib_core::model::Error::from)?;
	if let Err(error) = set_full_context_from_ctx_dbx(dbx, request_ctx).await {
		let _ = dbx.rollback_txn().await;
		return Err(error.into());
	}
	let loader = AuthorizationFactLoader::new(dbx, snapshot);
	if let Err(error) = loader.lock_and_verify_revisions().await {
		let _ = dbx.rollback_txn().await;
		return Err(map_fact_load_error(error));
	}
	let result = async {
		let context = loader
			.case_existing_for_verified_mutation(case_id, mutation_kind)
			.await
			.map_err(map_fact_load_error)?;
		let action = policy_registry()
			.context_action::<Existing<CaseResource>>(action_id)
			.ok_or_else(|| Error::AccessDenied {
				required_role: format!("registered {action_id} action"),
			})?;
		let permit = authorize_contextual_mutation(action, snapshot, context)
			.map_err(denied)?;
		let authorized_ctx =
			rls_ctx_for_authorized_mutation(request_ctx, snapshot, &permit)?;
		operation(&authorized_ctx, mm).await
	}
	.await;
	finish_fact_transaction(dbx, result).await
}

pub async fn with_authorized_case_child_read<T, F>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	mm: &ModelManager,
	case_id: Uuid,
	child_fingerprint: impl AsRef<str>,
	operation: F,
) -> Result<T>
where
	F: for<'ctx> FnOnce(
		&'ctx Ctx,
		&'ctx ModelManager,
	) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'ctx>>,
{
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(lib_core::model::Error::from)?;
	if let Err(error) = set_full_context_from_ctx_dbx(dbx, request_ctx).await {
		let _ = dbx.rollback_txn().await;
		return Err(error.into());
	}
	let result = async {
		let context = AuthorizationFactLoader::new(dbx, snapshot)
			.case_child(case_id, child_fingerprint)
			.await
			.map_err(map_fact_load_error)?;
		let action = policy_registry()
			.context_action::<Parent<CaseResource, CaseChildResource>>(
				"case.child.read",
			)
			.ok_or_else(|| Error::AccessDenied {
				required_role: "registered case.child.read action".to_string(),
			})?;
		let permit =
			authorize_contextual_read(action, snapshot, context).map_err(denied)?;
		let authorized_ctx =
			rls_ctx_for_authorized_read(request_ctx, snapshot, &permit)?;
		operation(&authorized_ctx, mm).await
	}
	.await;
	finish_fact_transaction(dbx, result).await
}

pub async fn with_authorized_case_child_mutation<T, F>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	mm: &ModelManager,
	case_id: Uuid,
	child_fingerprint: impl AsRef<str>,
	operation: F,
) -> Result<T>
where
	F: for<'ctx> FnOnce(
		&'ctx Ctx,
		&'ctx ModelManager,
	) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'ctx>>,
{
	let dbx = mm.dbx();
	dbx.begin_txn()
		.await
		.map_err(lib_core::model::Error::from)?;
	if let Err(error) = set_full_context_from_ctx_dbx(dbx, request_ctx).await {
		let _ = dbx.rollback_txn().await;
		return Err(error.into());
	}
	let loader = AuthorizationFactLoader::new(dbx, snapshot);
	if let Err(error) = loader.lock_and_verify_revisions().await {
		let _ = dbx.rollback_txn().await;
		return Err(map_fact_load_error(error));
	}
	let result = async {
		let context = loader
			.case_child_for_verified_mutation(case_id, child_fingerprint)
			.await
			.map_err(map_fact_load_error)?;
		let action = policy_registry()
			.context_action::<Parent<CaseResource, CaseChildResource>>(
				"case.child.update",
			)
			.ok_or_else(|| Error::AccessDenied {
				required_role: "registered case.child.update action".to_string(),
			})?;
		let permit = authorize_contextual_mutation(action, snapshot, context)
			.map_err(denied)?;
		let authorized_ctx =
			rls_ctx_for_authorized_mutation(request_ctx, snapshot, &permit)?;
		operation(&authorized_ctx, mm).await
	}
	.await;
	finish_fact_transaction(dbx, result).await
}

async fn finish_fact_transaction<T>(
	dbx: &lib_core::model::store::dbx::Dbx,
	result: Result<T>,
) -> Result<T> {
	match result {
		Ok(ctx) => {
			dbx.commit_txn()
				.await
				.map_err(lib_core::model::Error::from)?;
			Ok(ctx)
		}
		Err(error) => {
			let _ = dbx.rollback_txn().await;
			Err(error)
		}
	}
}

fn map_fact_load_error(error: AuthorizationFactLoadError) -> Error {
	match error {
		AuthorizationFactLoadError::Database(error) => {
			Error::Model(lib_core::model::Error::Dbx(error))
		}
		AuthorizationFactLoadError::FactNotFound { fact, id } => {
			Error::Model(lib_core::model::Error::EntityUuidNotFound {
				entity: fact,
				id,
			})
		}
		AuthorizationFactLoadError::SnapshotStale { .. } => {
			Error::PermissionDenied {
				required_permission: "AUTHORIZATION_SNAPSHOT_STALE".to_string(),
			}
		}
	}
}

pub fn rls_ctx_for_authorized_read<C: AuthorizationContext>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	permit: &AuthorizedRead<'_, C>,
) -> Result<Ctx> {
	rls_ctx_from_permit(request_ctx, snapshot, permit)
}

pub fn rls_ctx_for_authorized_mutation<C: AuthorizationContext>(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	permit: &AuthorizedMutation<'_, C>,
) -> Result<Ctx> {
	rls_ctx_from_permit(request_ctx, snapshot, permit)
}

fn rls_ctx_from_permit(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	permit: &impl PermitEvidence,
) -> Result<Ctx> {
	validate_request_binding(request_ctx, snapshot, permit)?;
	let Some(target_organization_id) = permit.target_organization_id() else {
		if snapshot.identity().is_platform_administrator() {
			return Ok(request_ctx.clone());
		}
		return Err(Error::AccessDenied {
			required_role: "authorization permit with target organization"
				.to_string(),
		});
	};
	if target_organization_id == request_ctx.organization_id() {
		return Ok(request_ctx.clone());
	}
	if !snapshot.identity().is_platform_administrator() {
		return Err(Error::AccessDenied {
			required_role: "platform administrator cross-organization permit"
				.to_string(),
		});
	}
	Ctx::new(
		request_ctx.user_id(),
		target_organization_id,
		ROLE_SYSTEM_ADMIN.to_string(),
	)
	.map(|ctx| {
		ctx.with_compliance(
			request_ctx.change_reason().map(ToString::to_string),
			request_ctx.e_signature_id(),
		)
		.with_change_category(request_ctx.change_category().map(ToString::to_string))
	})
	.map_err(|_| Error::BadRequest {
		message: "invalid authorized target organization context".to_string(),
	})
}

fn validate_request_binding(
	request_ctx: &Ctx,
	snapshot: &RequestAuthorizationSnapshot,
	permit: &impl RequestBoundPermit,
) -> Result<()> {
	if permit.principal_id() != request_ctx.user_id()
		|| permit.principal_id() != snapshot.principal_id()
		|| permit.organization_id() != snapshot.organization_id()
		|| request_ctx.organization_id() != snapshot.organization_id()
		|| permit.snapshot_version() != snapshot.version()
	{
		return Err(Error::AccessDenied {
			required_role: "authorization permit bound to this request".to_string(),
		});
	}
	Ok(())
}
