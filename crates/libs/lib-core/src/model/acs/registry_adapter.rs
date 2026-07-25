//! Compatibility adapter from the legacy menu-flag storage shape to the
//! registry-owned grant model.

use super::*;
use crate::authorization::{
	policy_registry, AdminMenuPrivilege, Availability, GrantUiField,
};

fn enabled(privilege: &AdminMenuPrivilege, field: GrantUiField) -> bool {
	match field {
		GrantUiField::CanRead => privilege.can_read,
		GrantUiField::CanEdit => privilege.can_edit,
		GrantUiField::CanReview => privilege.can_review,
		GrantUiField::CanLock => privilege.can_lock,
	}
}

fn push_unique(target: &mut Vec<Permission>, source: &[Permission]) {
	for permission in source {
		if !target.contains(permission) {
			target.push(*permission);
		}
	}
}

fn append_permissions_for_grant(target: &mut Vec<Permission>, id: &str) {
	match id {
		"home.notice.read" => push_unique(target, &[DASHBOARD_NOTICE_READ]),
		"home.notice.edit" => push_unique(target, &[DASHBOARD_NOTICE_UPDATE]),
		"home.workflow.read" | "case.workflow.read" => {
			push_unique(target, &[CASE_READ, CASE_LIST]);
		}
		"case.read" => push_unique(target, case_view_permissions()),
		"case.edit" => {
			push_unique(target, &[CASE_CREATE]);
			push_unique(target, profile_edit_permissions());
		}
		"case.review" => push_unique(target, &[CASE_APPROVE]),
		"case.lock" => push_unique(target, &[CASE_LOCK]),
		"info.read" => push_unique(
			target,
			&[
				PRESAVE_TEMPLATE_READ,
				PRESAVE_TEMPLATE_LIST,
				SENDER_INFORMATION_READ,
				SENDER_INFORMATION_LIST,
				RECEIVER_READ,
				RECEIVER_LIST,
				STUDY_INFORMATION_READ,
				STUDY_INFORMATION_LIST,
				NARRATIVE_READ,
				NARRATIVE_LIST,
			],
		),
		"info.edit" => push_unique(
			target,
			&[
				PRESAVE_TEMPLATE_CREATE,
				PRESAVE_TEMPLATE_UPDATE,
				PRESAVE_TEMPLATE_DELETE,
				SENDER_INFORMATION_CREATE,
				SENDER_INFORMATION_UPDATE,
				SENDER_INFORMATION_DELETE,
				RECEIVER_CREATE,
				RECEIVER_UPDATE,
				RECEIVER_DELETE,
				STUDY_INFORMATION_CREATE,
				STUDY_INFORMATION_UPDATE,
				STUDY_INFORMATION_DELETE,
				NARRATIVE_CREATE,
				NARRATIVE_UPDATE,
				NARRATIVE_DELETE,
			],
		),
		"import.history.read" => push_unique(target, &[XML_IMPORT_READ]),
		"import.execute" => push_unique(target, &[XML_IMPORT]),
		"submission.history.read" => push_unique(target, &[XML_EXPORT_READ]),
		"submission.execute" => push_unique(target, &[XML_EXPORT]),
		"admin.read" => push_unique(
			target,
			&[
				USER_READ,
				USER_LIST,
				ORG_READ,
				ORG_LIST,
				SETTINGS_READ,
				AUDIT_READ,
				AUDIT_LIST,
				TERMINOLOGY_READ,
			],
		),
		"admin.edit" => push_unique(
			target,
			&[
				USER_CREATE,
				USER_UPDATE,
				USER_DELETE,
				ORG_CREATE,
				ORG_UPDATE,
				ORG_DELETE,
				SETTINGS_UPDATE,
				TERMINOLOGY_IMPORT,
				TERMINOLOGY_APPROVE,
			],
		),
		_ => {}
	}
}

pub fn permissions_for_menu_privileges(
	privileges: &[AdminMenuPrivilege],
) -> Vec<Permission> {
	let registry = policy_registry();
	let mut grants = Vec::new();
	let normalized = privileges
		.iter()
		.filter_map(|privilege| {
			normalize_menu_privileges(std::slice::from_ref(privilege)).ok()
		})
		.flatten()
		.collect::<Vec<_>>();
	for privilege in &normalized {
		for grant in registry.grants().filter(|grant| {
			grant.availability == Availability::Implemented
				&& grant.ui_binding.menu_key == privilege.menu_key
				&& enabled(privilege, grant.ui_binding.field)
		}) {
			if !grants.iter().any(|id| id == &grant.id) {
				grants.push(grant.id.clone());
			}
		}
	}

	let Ok(grants) =
		registry.effective_grants(grants.iter().map(|grant| grant.as_str()))
	else {
		return Vec::new();
	};
	let mut permissions = Vec::new();
	for grant in grants {
		append_permissions_for_grant(&mut permissions, grant.as_str());
	}
	permissions
}
