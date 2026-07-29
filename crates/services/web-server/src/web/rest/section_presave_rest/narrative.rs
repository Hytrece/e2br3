use super::shared::*;

generate_single_row_presave_rest_fns! {
	Bmc: NarrativePresaveBmc,
	Entity: NarrativePresave,
	ForCreate: NarrativePresaveForCreate,
	ForUpdate: NarrativePresaveForUpdate,
	Row: narrative,
	Details: NarrativePresaveDetails,
	Rows: NarrativePresaveRows,
	CreateRequest: NarrativePresaveCreateRequest,
	CreateRows: NarrativePresaveCreateRows,
	UpdateRequest: NarrativePresaveUpdateRequest,
	UpdateRows: NarrativePresaveUpdateRows,
	CreateFn: create_narrative_presave,
	ListFn: list_narrative_presaves,
	GetFn: get_narrative_presave,
	UpdateFn: update_narrative_presave,
	DeleteFn: delete_narrative_presave,
	Kind: Narrative
}
