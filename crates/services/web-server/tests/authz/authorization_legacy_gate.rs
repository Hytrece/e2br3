use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("../../..")
		.canonicalize()
		.expect("workspace root must exist")
}

#[test]
fn user_administration_has_one_exact_permission_gate() {
	let root = workspace_root();
	let middleware = fs::read_to_string(
		root.join("crates/libs/lib-web/src/middleware/mw_permission.rs"),
	)
	.expect("permission middleware source must be readable");
	let rest_core =
		fs::read_to_string(root.join("crates/libs/lib-rest-core/src/lib.rs"))
			.expect("REST core source must be readable");
	let handlers = fs::read_to_string(
		root.join("crates/services/web-server/src/web/rest/user_rest/handlers.rs"),
	)
	.expect("user handlers source must be readable");

	assert!(
		!middleware.contains("struct RequireAdmin"),
		"legacy RequireAdmin extractor must not duplicate handler authorization"
	);
	assert!(
		!rest_core.contains("require_user_admin"),
		"broad user-admin gate must be replaced by exact USER_* authorization"
	);
	assert!(
		!handlers.contains("require_user_admin"),
		"user handlers must not layer a broad admin gate over exact permissions"
	);
	assert!(!handlers.contains("user_admin_db_ctx("));
	assert_eq!(
		handlers.matches("rls_ctx_for_authorized_").count(),
		5,
		"each user administration handler must derive DB scope from one permit"
	);
}

#[test]
fn legacy_admin_wrappers_and_dead_role_helpers_are_absent() {
	let root = workspace_root();
	let rest_core =
		fs::read_to_string(root.join("crates/libs/lib-rest-core/src/lib.rs"))
			.expect("REST core source must be readable");
	let ctx = fs::read_to_string(root.join("crates/libs/lib-core/src/ctx/mod.rs"))
		.expect("context source must be readable");
	let import = fs::read_to_string(
		root.join("crates/services/web-server/src/web/rest/import_rest.rs"),
	)
	.expect("import source must be readable");
	let presave = fs::read_to_string(root.join(
		"crates/services/web-server/src/web/rest/section_presave_rest/shared.rs",
	))
	.expect("presave source must be readable");

	assert!(
		!rest_core.contains("pub async fn is_admin"),
		"admin identity is synchronous and must not have a fake database wrapper"
	);
	assert!(
		!ctx.contains("pub fn can_modify"),
		"unused role-based modification shortcut must not bypass exact permissions"
	);
	assert!(!import.contains("lib_rest_core::is_admin"));
	assert!(!presave.contains("lib_rest_core::is_admin"));
}

#[test]
fn authorization_has_no_generic_admin_or_entitlement_middle_layer() {
	let root = workspace_root();
	let ctx = fs::read_to_string(root.join("crates/libs/lib-core/src/ctx/mod.rs"))
		.expect("context source must be readable");
	let rest_core =
		fs::read_to_string(root.join("crates/libs/lib-rest-core/src/lib.rs"))
			.expect("REST core source must be readable");
	let authorization_files = [
		"ids.rs",
		"definitions.rs",
		"registry.rs",
		"snapshot.rs",
		"kernel.rs",
		"contract.rs",
	];

	assert!(
		!ctx.contains("pub fn is_admin("),
		"Ctx must not expose a generic administrator authorization shortcut"
	);
	assert!(!rest_core.contains("pub fn user_admin_db_ctx("));
	assert!(!rest_core.contains("pub fn rls_ctx_for_user_admin("));

	for file in authorization_files {
		let source = fs::read_to_string(
			root.join("crates/libs/lib-core/src/authorization")
				.join(file),
		)
		.unwrap_or_else(|_| panic!("authorization source {file} must be readable"));
		assert!(
			!source.contains("EntitlementId")
				&& !source.contains("EntitlementDefinition")
				&& !source.contains("effective_entitlements("),
			"authorization source {file} still contains the entitlement middle layer"
		);
	}
}

#[test]
fn user_admin_rls_context_requires_authorization_permit_evidence() {
	let root = workspace_root();
	let rest_authorization =
		root.join("crates/libs/lib-rest-core/src/authorization.rs");
	let source = fs::read_to_string(&rest_authorization).unwrap_or_default();

	assert!(
		source.contains("AuthorizedRead") || source.contains("AuthorizedMutation"),
		"RLS context construction must consume kernel permit evidence"
	);
	assert!(
		source.contains("target_organization_id"),
		"permit-bound RLS context must verify the evaluated target organization"
	);
	assert!(
		source.contains("permit.snapshot_version() != snapshot.version()"),
		"RLS context construction must reject permits from a different policy snapshot"
	);
	assert!(
		source.contains(
			"request_ctx.organization_id() != snapshot.organization_id()"
		),
		"RLS context construction must bind the request organization to the snapshot"
	);
	assert!(
		source.contains("with_change_category"),
		"cross-organization RLS context must preserve the audit change category"
	);
}

#[test]
fn subject_actions_require_permit_bound_rls_context() {
	let root = workspace_root();
	let rest_authorization =
		root.join("crates/libs/lib-rest-core/src/authorization.rs");
	let source = fs::read_to_string(&rest_authorization).unwrap_or_default();

	assert!(
		source.contains("AuthorizedSubject"),
		"subject-action DB access must consume kernel permit evidence"
	);
	assert!(
		source.contains("pub fn rls_ctx_for_authorized_subject("),
		"subject actions need one explicit permit-to-RLS boundary"
	);
	assert!(
		source.contains("permit.snapshot_version() != snapshot.version()"),
		"subject permits must be bound to the current policy snapshot"
	);
}

#[test]
fn pdf_menu_privileges_are_not_part_of_the_legacy_permission_runtime() {
	let root = workspace_root();
	let menu_privileges =
		root.join("crates/libs/lib-core/src/authorization/menu_privileges.rs");
	let source = fs::read_to_string(&menu_privileges).unwrap_or_default();
	let assignment = fs::read_to_string(
		root.join("crates/libs/lib-core/src/model/authorization/assignment_repo.rs"),
	)
	.expect("assignment repository source must be readable");

	assert!(
		!source.is_empty(),
		"PDF menu privilege contract must live under authorization"
	);
	for legacy in [
		"Permission",
		"Resource",
		"has_permission",
		"dynamic_role",
		"permissions_for_menu_privileges",
	] {
		assert!(
			!source.contains(legacy),
			"menu privilege contract still depends on legacy runtime: {legacy}"
		);
	}
	assert!(
		assignment.contains("use crate::authorization::"),
		"normalized role persistence must import the PDF contract from authorization"
	);
	assert!(
		!assignment.contains("crate::model::acs"),
		"normalized role persistence must not depend on legacy ACS"
	);
}

#[test]
fn generated_case_routes_do_not_accept_legacy_permissions() {
	let root = workspace_root();
	let macros = fs::read_to_string(
		root.join("crates/libs/lib-rest-core/src/utils/macro_utils.rs"),
	)
	.expect("REST macro source must be readable");
	let case_macros = macros
		.split_once(
			"/// Generate CRUD REST handlers for a resource nested below a drug.",
		)
		.map(|(_, case_macros)| case_macros)
		.expect("Case macro boundary must exist");

	for legacy in [
		"require_permission",
		"RequirePermission",
		"check_permission",
		"legacy_permission_allowed",
		"PermCreate:",
		"PermRead:",
		"PermUpdate:",
		"PermDelete:",
		"PermList:",
	] {
		assert!(
			!case_macros.contains(legacy),
			"generated Case routes still accept legacy authorization input: {legacy}"
		);
	}
	assert!(
		case_macros.contains("AuthorizationSnapshotW"),
		"generated Case routes must consume the request authorization snapshot"
	);
	assert!(case_macros.contains("with_authorized_case_child_read"));
	assert!(case_macros.contains("with_authorized_case_child_mutation"));
}

#[test]
fn converted_case_route_modules_do_not_call_legacy_authorization() {
	let root = workspace_root();
	for file in [
		"case_rest.rs",
		"case_intake_rest.rs",
		"case_identifiers_rest.rs",
		"case_validation_rest.rs",
		"case_workflow_rest.rs",
		"drug_reaction_assessment_rest.rs",
		"message_header_rest.rs",
		"narrative_rest.rs",
		"narrative_sub_rest.rs",
		"parent_history_rest.rs",
		"patient_rest.rs",
		"patient_sub_rest.rs",
		"receiver_rest.rs",
		"relatedness_assessment_rest.rs",
		"safety_report_rest.rs",
		"safety_report_sub_rest.rs",
	] {
		let source = fs::read_to_string(
			root.join("crates/services/web-server/src/web/rest")
				.join(file),
		)
		.unwrap_or_else(|_| panic!("Case route source {file} must be readable"));
		for legacy in [
			"model::acs",
			"require_permission",
			"require_case_read_allowed",
			"require_case_write_allowed",
			"legacy_permission_allowed",
			"permission_subject()",
		] {
			assert!(
				!source.contains(legacy),
				"converted Case route {file} still calls legacy authorization: {legacy}"
			);
		}
		assert!(
			source.contains("AuthorizationSnapshotW")
				|| source.contains("generate_patient_child_rest_fns!"),
			"converted Case route {file} must consume the request snapshot"
		);
	}
}

#[test]
fn case_editor_routes_do_not_depend_on_legacy_permissions() {
	let root = workspace_root();
	let editor_root =
		root.join("crates/services/web-server/src/web/rest/case_editor_rest");
	for entry in
		fs::read_dir(&editor_root).expect("Case editor directory must exist")
	{
		let path = entry.expect("Case editor entry must be readable").path();
		if path.extension().and_then(|value| value.to_str()) != Some("rs") {
			continue;
		}
		let source = fs::read_to_string(&path)
			.unwrap_or_else(|_| panic!("{} must be readable", path.display()));
		for legacy in [
			"model::acs",
			"require_permission",
			"require_case_read_allowed",
			"require_case_write_allowed",
			"legacy_permission_allowed",
		] {
			assert!(
				!source.contains(legacy),
				"Case editor source {} still depends on legacy authorization: {legacy}",
				path.display()
			);
		}
	}
}

#[test]
fn role_api_has_one_canonical_metadata_shape() {
	let root = workspace_root();
	let source =
		fs::read_to_string(root.join(
			"crates/services/web-server/src/web/rest/permission_profile_rest.rs",
		))
		.expect("permission profile source must be readable");
	let model = fs::read_to_string(
		root.join("crates/libs/lib-core/src/model/permission_profile.rs"),
	)
	.expect("permission profile model source must be readable");
	let bootstrap =
		fs::read_to_string(root.join("db/bootstrap/01-safetydb-schema.sql"))
			.expect("bootstrap schema must be readable");
	let legacy_console = fs::read_to_string(root.join("web-folder/index.html"))
		.expect("legacy console source must be readable");

	for legacy in [
		"pub privilege_map:",
		"pub can_view:",
		"pub can_review:",
		"pub can_lock:",
		"pub can_admin:",
		"pub sponsor_admin_capable:",
		"pub is_builtin:",
		"pub is_editable:",
		"pub is_sponsor_admin:",
		"pub is_operational:",
		"fn role_summary_booleans(",
	] {
		assert!(
			!source.contains(legacy),
			"legacy role response field or derivation remains: {legacy}"
		);
	}
	assert!(source.contains("pub built_in: bool"));
	assert!(source.contains("pub editable: bool"));
	assert!(source.contains("pub privileges: Vec<AdminMenuPrivilege>"));
	assert!(!source.contains("sponsor_admin_capable"));
	assert!(!model.contains("sponsor_admin_capable"));
	assert!(!bootstrap.contains("sponsor_admin_capable"));
	assert!(!legacy_console.contains("sponsor_admin_capable"));
}

#[test]
fn user_role_metadata_does_not_turn_user_create_into_admin_identity() {
	let root = workspace_root();
	let dto = fs::read_to_string(
		root.join("crates/services/web-server/src/web/rest/user_rest/dto.rs"),
	)
	.expect("user DTO source must be readable");
	let validation = fs::read_to_string(
		root.join("crates/services/web-server/src/web/rest/user_rest/validation.rs"),
	)
	.expect("user validation source must be readable");
	let openapi =
		fs::read_to_string(root.join("crates/services/web-server/src/openapi.rs"))
			.expect("OpenAPI source must be readable");

	assert!(!dto.contains("pub can_admin:"));
	assert!(!validation.contains("has_permission(permission_subject, USER_CREATE)"));
	assert!(!openapi.contains("\tcan_admin: bool,"));
}

#[test]
fn built_in_role_metadata_has_one_backend_source() {
	let root = workspace_root();
	let permission_profiles =
		fs::read_to_string(root.join(
			"crates/services/web-server/src/web/rest/permission_profile_rest.rs",
		))
		.expect("permission profile source must be readable");
	let user_validation = fs::read_to_string(
		root.join("crates/services/web-server/src/web/rest/user_rest/validation.rs"),
	)
	.expect("user validation source must be readable");

	for duplicate_label in [
		"System Administrator",
		"Sponsor Administrator (CRO)",
		"Sponsor Administrator (Pharmaceutical Company)",
		"CRO Sponsor Administrator",
		"Company Sponsor Administrator",
	] {
		assert!(!permission_profiles.contains(duplicate_label));
	}
	for duplicate_display_expression in [
		"\"System Administrator\".to_string()",
		"\"Sponsor Administrator (CRO)\".to_string()",
		"\"Sponsor Administrator (Pharmaceutical Company)\".to_string()",
	] {
		assert!(!user_validation.contains(duplicate_display_expression));
	}
	assert!(permission_profiles.contains("built_in_role_metadata("));
	assert!(user_validation.contains("built_in_role_metadata("));
}

#[test]
fn legacy_console_does_not_call_removed_role_api() {
	let console = fs::read_to_string(workspace_root().join("web-folder/index.html"))
		.expect("legacy console source must be readable");
	assert!(!console.contains("/api/admin/roles"));
	assert!(!console.contains("function loadRoles"));
	assert!(!console.contains("function createRole"));
}

#[test]
fn production_routes_do_not_evaluate_legacy_permissions_directly() {
	let root = workspace_root();
	for path in [
		"crates/libs/lib-rest-core/src/lib.rs",
		"crates/libs/lib-web/src/middleware/mw_permission.rs",
		"crates/services/web-server/src/web/rest/admin_settings_rest.rs",
		"crates/services/web-server/src/web/rest/audit_rest.rs",
		"crates/services/web-server/src/web/rest/case_rest.rs",
		"crates/services/web-server/src/web/rest/user_rest.rs",
		"crates/services/web-server/src/web/rest/user_rest/handlers.rs",
	] {
		let source = fs::read_to_string(root.join(path)).unwrap_or_else(|_| {
			panic!("authorization source {path} must be readable")
		});
		assert!(
			!source.contains("has_permission("),
			"{path} bypasses the authorization kernel compatibility entry point"
		);
	}
}

#[test]
fn role_reactivation_uses_the_restore_policy_action() {
	let source =
		fs::read_to_string(workspace_root().join(
			"crates/services/web-server/src/web/rest/permission_profile_rest.rs",
		))
		.expect("permission profile source must be readable");
	assert!(
		source.contains("data.active == Some(true)")
			&& source.contains("\"role.restore\""),
		"an explicit role reactivation must be authorized and audited as role.restore"
	);
}
