pub(crate) mod patch;

pub(crate) use patch::patch_e_reactions_for_authority;
pub(crate) use patch::patch_g_drugs_for_authority;
pub use patch::{
	patch_c_safety_report, patch_d_patient, patch_e_reactions, patch_f_test_results,
	patch_g_drugs, patch_h_narrative, CSafetyReportPatch, DPatientDeathCausePatch,
	DPatientPatch,
};
pub(crate) use patch::{
	reorder_investigation_event_children, reorder_patient_player_children,
};
