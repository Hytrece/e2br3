use super::shared::*;

generate_single_row_presave_rest_fns! {
	Bmc: ReporterPresaveBmc,
	Entity: ReporterPresave,
	ForCreate: ReporterPresaveForCreate,
	ForUpdate: ReporterPresaveForUpdate,
	Row: reporter,
	Details: ReporterPresaveDetails,
	Rows: ReporterPresaveRows,
	CreateRequest: ReporterPresaveCreateRequest,
	CreateRows: ReporterPresaveCreateRows,
	UpdateRequest: ReporterPresaveUpdateRequest,
	UpdateRows: ReporterPresaveUpdateRows,
	CreateFn: create_reporter_presave,
	ListFn: list_reporter_presaves,
	GetFn: get_reporter_presave,
	UpdateFn: update_reporter_presave,
	DeleteFn: delete_reporter_presave,
	Kind: Reporter
}
