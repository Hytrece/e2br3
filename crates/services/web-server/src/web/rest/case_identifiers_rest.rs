// Case Identifiers REST endpoints (C.1.9.r and C.1.10.r)

use axum::http::StatusCode;
use lib_core::model::case_identifiers::{
	LinkedReportNumber, LinkedReportNumberBmc, LinkedReportNumberFilter,
	LinkedReportNumberForCreate, LinkedReportNumberForUpdate, OtherCaseIdentifier,
	OtherCaseIdentifierBmc, OtherCaseIdentifierFilter, OtherCaseIdentifierForCreate,
	OtherCaseIdentifierForUpdate,
};
use lib_core::model::{self, ModelManager};
use lib_rest_core::Result;
use uuid::Uuid;

async fn case_id_for_case(
	_ctx: &lib_core::ctx::Ctx,
	_mm: &ModelManager,
	case_id: Uuid,
) -> Result<Uuid> {
	Ok(case_id)
}

async fn ensure_case_child_scope(
	_ctx: &lib_core::ctx::Ctx,
	_mm: &ModelManager,
	case_id: Uuid,
	entity_case_id: Uuid,
	entity_id: Uuid,
	entity: &'static str,
) -> Result<()> {
	if case_id != entity_case_id {
		return Err(model::Error::EntityUuidNotFound {
			entity,
			id: entity_id,
		}
		.into());
	}
	Ok(())
}

// -- Other Case Identifiers (C.1.9.r)

lib_rest_core::generate_patient_child_rest_fns! {
	Bmc: OtherCaseIdentifierBmc,
	Entity: OtherCaseIdentifier,
	ForCreate: OtherCaseIdentifierForCreate,
	ForUpdate: OtherCaseIdentifierForUpdate,
	Filter: OtherCaseIdentifierFilter,
	CreateFn: create_other_case_identifier,
	ListFn: list_other_case_identifiers,
	GetFn: get_other_case_identifier,
	UpdateFn: update_other_case_identifier,
	DeleteFn: delete_other_case_identifier,
	RestoreFn: restore_other_case_identifier,
	ParentField: case_id,
	ResolveParentFn: case_id_for_case,
	ScopeFn: ensure_case_child_scope,
	EntityName: "other_case_identifiers",
	DeleteResult: StatusCode,
	DeleteResponse: no_content
}

// -- Linked Report Numbers (C.1.10.r)

lib_rest_core::generate_patient_child_rest_fns! {
	Bmc: LinkedReportNumberBmc,
	Entity: LinkedReportNumber,
	ForCreate: LinkedReportNumberForCreate,
	ForUpdate: LinkedReportNumberForUpdate,
	Filter: LinkedReportNumberFilter,
	CreateFn: create_linked_report_number,
	ListFn: list_linked_report_numbers,
	GetFn: get_linked_report_number,
	UpdateFn: update_linked_report_number,
	DeleteFn: delete_linked_report_number,
	RestoreFn: restore_linked_report_number,
	ParentField: case_id,
	ResolveParentFn: case_id_for_case,
	ScopeFn: ensure_case_child_scope,
	EntityName: "linked_report_numbers",
	DeleteResult: StatusCode,
	DeleteResponse: no_content
}
