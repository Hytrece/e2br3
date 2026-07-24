use crate::authorization::{
	CaseResource, Collection, ContextSnapshot, EnforcedScopeFilter,
	EvaluatedContext, Existing, LockedMutationContext, RequestAuthorizationSnapshot,
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
	ChildEdit,
	WorkflowTransition,
}

#[derive(Debug, FromRow)]
struct CaseFacts {
	organization_id: Uuid,
	status: String,
	status_before_lock: Option<String>,
	sender_identifiers: Vec<String>,
	product_identifiers: Vec<String>,
	study_identifiers: Vec<String>,
	has_blinded_data: bool,
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
		let facts = self.load_case_facts(case_id, true).await?;
		Ok(LockedMutationContext::new(case_evaluated(
			self.snapshot,
			case_id,
			&facts,
			case_lifecycle_allows(&facts, kind),
		)))
	}

	async fn lock_and_verify_revisions(
		&self,
	) -> Result<(), AuthorizationFactLoadError> {
		let (organization_revision, principal_revision) = self
			.dbx
			.fetch_one(
				sqlx::query_as::<_, (i64, i64)>(
					"SELECT o.revision, p.revision
					   FROM organization_policy_state o
					   JOIN principal_authorization_state p
					     ON p.organization_id = o.organization_id
					  WHERE o.organization_id = $1 AND p.user_id = $2
					  FOR UPDATE OF o, p",
				)
				.bind(self.snapshot.organization_id())
				.bind(self.snapshot.principal_id()),
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
			       c.status_before_lock,
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
					&& facts.status_before_lock.as_deref().is_some_and(|previous| {
						matches!(
							previous.trim().to_ascii_lowercase().as_str(),
							"draft" | "reviewed" | "validated"
						)
					})
		}
		CaseMutationKind::Delete => {
			!matches!(status.as_str(), "deleted" | "archived")
		}
		CaseMutationKind::Validate => {
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
