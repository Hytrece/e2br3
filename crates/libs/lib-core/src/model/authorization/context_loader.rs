use crate::authorization::{
	AuditLogResource, BuiltInIdentityKind, CaseAuditTrailResource,
	CaseChildResource, CaseCreateProposal, CaseResource, Collection,
	ContextSnapshot, EnforcedScopeFilter, EvaluatedContext, Existing,
	ImportHistoryResource, LockedMutationContext, NoticeResource, Parent, Proposed,
	RequestAuthorizationSnapshot, ResourceSet, SettingsResource, SubmissionResource,
	TerminologyImportProposal, TerminologyResource, XmlImportBatchProposal,
};
use crate::model::store::dbx::Dbx;
use sqlx::FromRow;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthorizationFactLoadError {
	Database(crate::model::store::dbx::Error),
	FactNotFound {
		fact: &'static str,
		id: Uuid,
	},
	SnapshotStale {
		snapshot_organization_revision: i64,
		current_organization_revision: i64,
		snapshot_principal_revision: i64,
		current_principal_revision: i64,
	},
}

impl Display for AuthorizationFactLoadError {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		write!(formatter, "{self:?}")
	}
}

impl std::error::Error for AuthorizationFactLoadError {}

impl From<crate::model::store::dbx::Error> for AuthorizationFactLoadError {
	fn from(error: crate::model::store::dbx::Error) -> Self {
		Self::Database(error)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseMutationKind {
	Edit,
	ReviewToggle,
	LockToggle,
	Delete,
	Validate,
	Submission,
	ChildEdit,
	WorkflowTransition,
}

#[derive(Debug, FromRow)]
struct CaseFacts {
	organization_id: Uuid,
	status: String,
	sender_identifiers: Vec<String>,
	product_identifiers: Vec<String>,
	study_identifiers: Vec<String>,
	has_blinded_data: bool,
}

#[derive(Debug, FromRow)]
struct SubmissionParent {
	case_id: Uuid,
}

#[derive(Debug, FromRow)]
struct ImportHistoryFacts {
	organization_id: Uuid,
	case_id: Option<Uuid>,
	uploaded_by: Uuid,
}

#[derive(Debug, FromRow)]
struct ExportHistoryParent {
	case_id: Uuid,
}

pub struct AuthorizationFactLoader<'tx> {
	dbx: &'tx Dbx,
	snapshot: &'tx RequestAuthorizationSnapshot,
}

impl<'tx> AuthorizationFactLoader<'tx> {
	pub fn new(dbx: &'tx Dbx, snapshot: &'tx RequestAuthorizationSnapshot) -> Self {
		Self { dbx, snapshot }
	}

	pub fn case_collection(&self) -> ContextSnapshot<'tx, Collection<CaseResource>> {
		ContextSnapshot::new(EvaluatedContext {
			organization_id: Some(self.snapshot.organization_id()),
			target_fingerprint: format!("cases:{}", self.snapshot.organization_id()),
			within_principal_scope: true,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: Some(scope_filter(self.snapshot)),
		})
	}

	pub fn settings_existing(
		&self,
	) -> ContextSnapshot<'tx, Existing<SettingsResource>> {
		ContextSnapshot::new(organization_resource_evaluated(
			self.snapshot.organization_id(),
			"settings",
		))
	}

	pub async fn settings_for_mutation(
		&self,
	) -> Result<
		LockedMutationContext<'tx, Existing<SettingsResource>>,
		AuthorizationFactLoadError,
	> {
		self.lock_and_verify_revisions().await?;
		Ok(LockedMutationContext::new(organization_resource_evaluated(
			self.snapshot.organization_id(),
			"settings",
		)))
	}

	pub async fn notice_for_mutation(
		&self,
	) -> Result<
		LockedMutationContext<'tx, Existing<NoticeResource>>,
		AuthorizationFactLoadError,
	> {
		self.lock_and_verify_revisions().await?;
		Ok(LockedMutationContext::new(organization_resource_evaluated(
			self.snapshot.organization_id(),
			"notice",
		)))
	}

	pub fn audit_log_collection(
		&self,
	) -> ContextSnapshot<'tx, Collection<AuditLogResource>> {
		ContextSnapshot::new(organization_resource_evaluated(
			self.snapshot.organization_id(),
			"audit-logs",
		))
	}

	pub fn terminology_collection(
		&self,
	) -> ContextSnapshot<'tx, Collection<TerminologyResource>> {
		ContextSnapshot::new(organization_resource_evaluated(
			self.snapshot.organization_id(),
			"terminology",
		))
	}

	pub async fn terminology_import_for_mutation(
		&self,
		fingerprint: impl AsRef<str>,
	) -> Result<
		LockedMutationContext<'tx, Proposed<TerminologyImportProposal>>,
		AuthorizationFactLoadError,
	> {
		self.lock_and_verify_revisions().await?;
		Ok(LockedMutationContext::new(EvaluatedContext {
			organization_id: Some(self.snapshot.organization_id()),
			target_fingerprint: format!(
				"terminology:{}:{}",
				self.snapshot.organization_id(),
				fingerprint.as_ref()
			),
			within_principal_scope: false,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: None,
		}))
	}

	pub async fn case_resource_set(
		&self,
		case_ids: &[Uuid],
	) -> Result<
		ContextSnapshot<'tx, ResourceSet<CaseResource>>,
		AuthorizationFactLoadError,
	> {
		let mut ids = case_ids.to_vec();
		ids.sort_unstable();
		ids.dedup();
		let mut every_target_authorized = true;
		for case_id in &ids {
			let facts = self.load_case_facts(*case_id, false).await?;
			let same_organization = facts.organization_id
				== self.snapshot.organization_id()
				|| self.snapshot.identity().is_platform_administrator();
			if !same_organization || !case_within_scope(self.snapshot, &facts) {
				every_target_authorized = false;
			}
		}
		Ok(ContextSnapshot::new(EvaluatedContext {
			organization_id: Some(self.snapshot.organization_id()),
			target_fingerprint: format!(
				"case-export:{}",
				ids.iter()
					.map(Uuid::to_string)
					.collect::<Vec<_>>()
					.join(",")
			),
			within_principal_scope: false,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized,
			enforced_scope_filter: None,
		}))
	}

	pub async fn export_history_parent_case_id(
		&self,
		history_id: Uuid,
	) -> Result<Uuid, AuthorizationFactLoadError> {
		self.dbx
			.fetch_optional(
				sqlx::query_as::<_, ExportHistoryParent>(
					"SELECT case_id
					   FROM xml_export_history
					  WHERE id = $1",
				)
				.bind(history_id),
			)
			.await?
			.map(|row| row.case_id)
			.ok_or(AuthorizationFactLoadError::FactNotFound {
				fact: "export_history",
				id: history_id,
			})
	}

	pub fn submission_collection(
		&self,
	) -> ContextSnapshot<'tx, Collection<SubmissionResource>> {
		ContextSnapshot::new(EvaluatedContext {
			organization_id: Some(self.snapshot.organization_id()),
			target_fingerprint: format!(
				"submissions:{}",
				self.snapshot.organization_id()
			),
			within_principal_scope: true,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: Some(scope_filter(self.snapshot)),
		})
	}

	pub fn import_history_collection(
		&self,
	) -> ContextSnapshot<'tx, Collection<ImportHistoryResource>> {
		ContextSnapshot::new(EvaluatedContext {
			organization_id: Some(self.snapshot.organization_id()),
			target_fingerprint: format!(
				"import-history:{}",
				self.snapshot.organization_id()
			),
			within_principal_scope: true,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: Some(scope_filter(self.snapshot)),
		})
	}

	pub async fn import_history_existing(
		&self,
		history_id: Uuid,
	) -> Result<
		ContextSnapshot<'tx, Existing<ImportHistoryResource>>,
		AuthorizationFactLoadError,
	> {
		let facts = self
			.dbx
			.fetch_optional(
				sqlx::query_as::<_, ImportHistoryFacts>(
					"SELECT u.organization_id, h.case_id, h.uploaded_by
					   FROM xml_import_history h
					   JOIN users u ON u.id = h.uploaded_by
					  WHERE h.id = $1",
				)
				.bind(history_id),
			)
			.await?
			.ok_or(AuthorizationFactLoadError::FactNotFound {
				fact: "import_history",
				id: history_id,
			})?;
		let within_principal_scope = match facts.case_id {
			Some(case_id) => {
				let case = self.load_case_facts(case_id, false).await?;
				case_within_scope(self.snapshot, &case)
			}
			None => {
				facts.uploaded_by == self.snapshot.principal_id()
					|| matches!(
						self.snapshot.identity().built_in_kind(),
						Some(
							BuiltInIdentityKind::PlatformAdministrator
								| BuiltInIdentityKind::SponsorCroAdministrator
								| BuiltInIdentityKind::SponsorCompanyAdministrator
						)
					)
			}
		};
		Ok(ContextSnapshot::new(EvaluatedContext {
			organization_id: Some(facts.organization_id),
			target_fingerprint: format!("import-history:{history_id}"),
			within_principal_scope,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: None,
		}))
	}

	pub async fn xml_import_batch_for_mutation(
		&self,
		fingerprint: impl AsRef<str>,
	) -> Result<
		LockedMutationContext<'tx, Proposed<XmlImportBatchProposal>>,
		AuthorizationFactLoadError,
	> {
		self.lock_and_verify_revisions().await?;
		Ok(LockedMutationContext::new(EvaluatedContext {
			organization_id: Some(self.snapshot.organization_id()),
			target_fingerprint: format!(
				"xml-import:{}:{}",
				self.snapshot.organization_id(),
				fingerprint.as_ref()
			),
			within_principal_scope: true,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: None,
		}))
	}

	pub async fn submission_existing(
		&self,
		submission_id: Uuid,
	) -> Result<
		ContextSnapshot<'tx, Existing<SubmissionResource>>,
		AuthorizationFactLoadError,
	> {
		let parent = self
			.dbx
			.fetch_optional(
				sqlx::query_as::<_, SubmissionParent>(
					"SELECT cs.case_id
			   FROM case_submissions cs
			  WHERE cs.id = $1",
				)
				.bind(submission_id),
			)
			.await?
			.ok_or(AuthorizationFactLoadError::FactNotFound {
				fact: "submission",
				id: submission_id,
			})?;
		let case = self.load_case_facts(parent.case_id, false).await?;
		Ok(ContextSnapshot::new(EvaluatedContext {
			organization_id: Some(case.organization_id),
			target_fingerprint: format!(
				"submission:{submission_id}:case:{}",
				parent.case_id
			),
			within_principal_scope: case_within_scope(self.snapshot, &case),
			lifecycle_compatible: !matches!(
				case.status.trim().to_ascii_lowercase().as_str(),
				"deleted" | "archived"
			),
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: None,
		}))
	}

	pub async fn submission_parent_case_for_mutation(
		&self,
		submission_id: Uuid,
	) -> Result<
		LockedMutationContext<'tx, Existing<CaseResource>>,
		AuthorizationFactLoadError,
	> {
		self.lock_and_verify_revisions().await?;
		let parent = self
			.dbx
			.fetch_optional(
				sqlx::query_as::<_, SubmissionParent>(
					"SELECT cs.case_id
					   FROM case_submissions cs
					  WHERE cs.id = $1
					  FOR UPDATE OF cs",
				)
				.bind(submission_id),
			)
			.await?
			.ok_or(AuthorizationFactLoadError::FactNotFound {
				fact: "submission",
				id: submission_id,
			})?;
		let case = self.load_case_facts(parent.case_id, true).await?;
		Ok(LockedMutationContext::new(EvaluatedContext {
			organization_id: Some(case.organization_id),
			target_fingerprint: format!(
				"case:{}:submission:{submission_id}",
				parent.case_id,
			),
			within_principal_scope: case_within_scope(self.snapshot, &case),
			lifecycle_compatible: !matches!(
				case.status.trim().to_ascii_lowercase().as_str(),
				"deleted" | "archived"
			),
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: None,
		}))
	}

	pub fn case_create_for_verified_mutation(
		&self,
		organization_id: Uuid,
	) -> LockedMutationContext<'tx, Proposed<CaseCreateProposal>> {
		let within_principal_scope = organization_id
			== self.snapshot.organization_id()
			|| self.snapshot.identity().is_platform_administrator();
		LockedMutationContext::new(EvaluatedContext {
			organization_id: Some(organization_id),
			target_fingerprint: format!("case:new:{organization_id}"),
			within_principal_scope,
			lifecycle_compatible: false,
			parent_authorized: false,
			every_target_authorized: false,
			enforced_scope_filter: None,
		})
	}

	pub async fn case_existing(
		&self,
		case_id: Uuid,
	) -> Result<
		ContextSnapshot<'tx, Existing<CaseResource>>,
		AuthorizationFactLoadError,
	> {
		let facts = self.load_case_facts(case_id, false).await?;
		Ok(ContextSnapshot::new(case_evaluated(
			self.snapshot,
			case_id,
			&facts,
			false,
		)))
	}

	pub async fn case_existing_for_mutation(
		&self,
		case_id: Uuid,
		kind: CaseMutationKind,
	) -> Result<
		LockedMutationContext<'tx, Existing<CaseResource>>,
		AuthorizationFactLoadError,
	> {
		self.lock_and_verify_revisions().await?;
		self.case_existing_for_verified_mutation(case_id, kind)
			.await
	}

	pub async fn case_existing_for_verified_mutation(
		&self,
		case_id: Uuid,
		kind: CaseMutationKind,
	) -> Result<
		LockedMutationContext<'tx, Existing<CaseResource>>,
		AuthorizationFactLoadError,
	> {
		let facts = self.load_case_facts(case_id, true).await?;
		Ok(LockedMutationContext::new(case_evaluated(
			self.snapshot,
			case_id,
			&facts,
			case_lifecycle_allows(&facts, kind),
		)))
	}

	pub async fn case_child(
		&self,
		case_id: Uuid,
		child_fingerprint: impl AsRef<str>,
	) -> Result<
		ContextSnapshot<'tx, Parent<CaseResource, CaseChildResource>>,
		AuthorizationFactLoadError,
	> {
		let facts = self.load_case_facts(case_id, false).await?;
		Ok(ContextSnapshot::new(case_child_evaluated(
			self.snapshot,
			case_id,
			child_fingerprint.as_ref(),
			&facts,
			false,
		)))
	}

	pub async fn case_audit_trail(
		&self,
		case_id: Uuid,
	) -> Result<
		ContextSnapshot<'tx, Parent<CaseResource, CaseAuditTrailResource>>,
		AuthorizationFactLoadError,
	> {
		let facts = self.load_case_facts(case_id, false).await?;
		Ok(ContextSnapshot::new(case_child_evaluated(
			self.snapshot,
			case_id,
			"audit-trail",
			&facts,
			false,
		)))
	}

	pub async fn case_child_for_mutation(
		&self,
		case_id: Uuid,
		child_fingerprint: impl AsRef<str>,
	) -> Result<
		LockedMutationContext<'tx, Parent<CaseResource, CaseChildResource>>,
		AuthorizationFactLoadError,
	> {
		self.lock_and_verify_revisions().await?;
		self.case_child_for_verified_mutation(case_id, child_fingerprint)
			.await
	}

	pub async fn case_child_for_verified_mutation(
		&self,
		case_id: Uuid,
		child_fingerprint: impl AsRef<str>,
	) -> Result<
		LockedMutationContext<'tx, Parent<CaseResource, CaseChildResource>>,
		AuthorizationFactLoadError,
	> {
		let facts = self.load_case_facts(case_id, true).await?;
		Ok(LockedMutationContext::new(case_child_evaluated(
			self.snapshot,
			case_id,
			child_fingerprint.as_ref(),
			&facts,
			case_lifecycle_allows(&facts, CaseMutationKind::ChildEdit),
		)))
	}

	pub async fn lock_and_verify_revisions(
		&self,
	) -> Result<(), AuthorizationFactLoadError> {
		let (organization_revision, principal_revision) = self
			.dbx
			.fetch_one(
				sqlx::query_as::<_, (i64, i64)>(
					"SELECT organization_revision, principal_revision
					   FROM authz_lock_policy_revisions($1, $2)",
				)
				.bind(self.snapshot.principal_id())
				.bind(self.snapshot.organization_id()),
			)
			.await?;
		ensure_current_revisions(
			self.snapshot,
			organization_revision,
			principal_revision,
		)
	}

	async fn load_case_facts(
		&self,
		case_id: Uuid,
		for_update: bool,
	) -> Result<CaseFacts, AuthorizationFactLoadError> {
		let lock_clause = if for_update { "FOR UPDATE OF c" } else { "" };
		let sql = format!(
			r#"
			SELECT c.organization_id,
			       c.status,
			       COALESCE(
			       	(SELECT array_agg(DISTINCT s.source_sender_presave_id::text)
			       	   FROM sender_information s
			       	  WHERE s.case_id = c.id
			       	    AND s.source_sender_presave_id IS NOT NULL),
			       	ARRAY[]::text[]
			       ) AS sender_identifiers,
			       COALESCE(
			       	(SELECT array_agg(DISTINCT d.source_product_presave_id::text)
			       	   FROM drug_information d
			       	  WHERE d.case_id = c.id
			       	    AND d.source_product_presave_id IS NOT NULL),
			       	ARRAY[]::text[]
			       ) AS product_identifiers,
			       COALESCE(
			       	(SELECT array_agg(DISTINCT s.source_study_presave_id::text)
			       	   FROM study_information s
			       	  WHERE s.case_id = c.id
			       	    AND s.source_study_presave_id IS NOT NULL),
			       	ARRAY[]::text[]
			       ) AS study_identifiers,
			       EXISTS(
			       	SELECT 1 FROM drug_information d
			       	 WHERE d.case_id = c.id
			       	   AND d.investigational_product_blinded = TRUE
			       ) AS has_blinded_data
			  FROM cases c
			 WHERE c.id = $1
			 {lock_clause}
			"#
		);
		self.dbx
			.fetch_optional(sqlx::query_as::<_, CaseFacts>(&sql).bind(case_id))
			.await?
			.ok_or(AuthorizationFactLoadError::FactNotFound {
				fact: "case",
				id: case_id,
			})
	}
}

fn scope_filter(snapshot: &RequestAuthorizationSnapshot) -> EnforcedScopeFilter {
	EnforcedScopeFilter::new(
		snapshot.scope().sender_ids().to_vec(),
		snapshot.scope().product_ids().to_vec(),
		snapshot.scope().study_ids().to_vec(),
		snapshot.scope().blind_allowed(),
	)
}

fn organization_resource_evaluated(
	organization_id: Uuid,
	resource: &str,
) -> EvaluatedContext {
	EvaluatedContext {
		organization_id: Some(organization_id),
		target_fingerprint: format!("{resource}:{organization_id}"),
		within_principal_scope: false,
		lifecycle_compatible: false,
		parent_authorized: false,
		every_target_authorized: false,
		enforced_scope_filter: None,
	}
}

fn case_evaluated(
	snapshot: &RequestAuthorizationSnapshot,
	case_id: Uuid,
	facts: &CaseFacts,
	lifecycle_compatible: bool,
) -> EvaluatedContext {
	EvaluatedContext {
		organization_id: Some(facts.organization_id),
		target_fingerprint: format!("case:{case_id}:{}", facts.status),
		within_principal_scope: case_within_scope(snapshot, facts),
		lifecycle_compatible,
		parent_authorized: false,
		every_target_authorized: false,
		enforced_scope_filter: None,
	}
}

fn case_child_evaluated(
	snapshot: &RequestAuthorizationSnapshot,
	case_id: Uuid,
	child_fingerprint: &str,
	facts: &CaseFacts,
	lifecycle_compatible: bool,
) -> EvaluatedContext {
	let same_organization = facts.organization_id == snapshot.organization_id()
		|| snapshot.identity().is_platform_administrator();
	EvaluatedContext {
		organization_id: Some(facts.organization_id),
		target_fingerprint: format!("case:{case_id}:child:{child_fingerprint}"),
		within_principal_scope: false,
		lifecycle_compatible,
		parent_authorized: same_organization && case_within_scope(snapshot, facts),
		every_target_authorized: false,
		enforced_scope_filter: None,
	}
}

fn case_within_scope(
	snapshot: &RequestAuthorizationSnapshot,
	facts: &CaseFacts,
) -> bool {
	scope_allows(snapshot.scope().sender_ids(), &facts.sender_identifiers)
		&& scope_allows(snapshot.scope().product_ids(), &facts.product_identifiers)
		&& scope_allows(snapshot.scope().study_ids(), &facts.study_identifiers)
		&& (!facts.has_blinded_data || snapshot.scope().blind_allowed())
}

fn scope_allows(assigned: &[String], available: &[String]) -> bool {
	if assigned.is_empty() || available.is_empty() {
		return true;
	}
	available.iter().any(|candidate| {
		assigned
			.iter()
			.any(|assigned| assigned.eq_ignore_ascii_case(candidate))
	})
}

fn case_lifecycle_allows(facts: &CaseFacts, kind: CaseMutationKind) -> bool {
	let status = facts.status.trim().to_ascii_lowercase();
	match kind {
		CaseMutationKind::Edit | CaseMutationKind::ChildEdit => status == "draft",
		CaseMutationKind::ReviewToggle => {
			matches!(status.as_str(), "draft" | "reviewed" | "validated")
		}
		CaseMutationKind::LockToggle => {
			matches!(status.as_str(), "draft" | "reviewed" | "validated")
				|| status == "locked"
		}
		CaseMutationKind::Delete => {
			!matches!(status.as_str(), "deleted" | "archived")
		}
		CaseMutationKind::Validate => {
			!matches!(status.as_str(), "locked" | "deleted" | "archived")
		}
		CaseMutationKind::Submission => {
			!matches!(status.as_str(), "locked" | "deleted" | "archived")
		}
		CaseMutationKind::WorkflowTransition => status != "locked",
	}
}

fn ensure_current_revisions(
	snapshot: &RequestAuthorizationSnapshot,
	current_organization_revision: i64,
	current_principal_revision: i64,
) -> Result<(), AuthorizationFactLoadError> {
	let version = snapshot.version();
	if version.organization_revision() != current_organization_revision
		|| version.principal_revision() != current_principal_revision
	{
		return Err(AuthorizationFactLoadError::SnapshotStale {
			snapshot_organization_revision: version.organization_revision(),
			current_organization_revision,
			snapshot_principal_revision: version.principal_revision(),
			current_principal_revision,
		});
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::authorization::{
		BuiltInIdentityKind, GrantId, IdentityTraits, PolicySnapshotVersion,
		PrincipalScope, RequestAuthorizationSnapshot,
	};
	use std::collections::BTreeSet;
	use time::OffsetDateTime;
	use uuid::Uuid;

	fn snapshot(
		organization_revision: i64,
		principal_revision: i64,
	) -> RequestAuthorizationSnapshot {
		let organization_id = Uuid::new_v4();
		RequestAuthorizationSnapshot::new(
			Uuid::new_v4(),
			organization_id,
			Uuid::new_v4(),
			IdentityTraits::new(Some(BuiltInIdentityKind::OperationalUser)),
			BTreeSet::<GrantId>::new(),
			PrincipalScope::new(Vec::new(), Vec::new(), Vec::new(), false, None),
			PolicySnapshotVersion::new(
				"a".repeat(64),
				organization_id,
				organization_revision,
				principal_revision,
			),
			OffsetDateTime::now_utc(),
			None,
			"test".into(),
		)
	}

	#[test]
	fn mutation_facts_reject_a_stale_policy_snapshot() {
		let snapshot = snapshot(4, 7);

		let error = ensure_current_revisions(&snapshot, 5, 7)
			.expect_err("changed organization revision must be stale");

		assert!(matches!(
			error,
			AuthorizationFactLoadError::SnapshotStale {
				snapshot_organization_revision: 4,
				current_organization_revision: 5,
				snapshot_principal_revision: 7,
				current_principal_revision: 7,
			}
		));
	}
}
