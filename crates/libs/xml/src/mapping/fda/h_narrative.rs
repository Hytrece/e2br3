// FDA mapping for Section H (Narrative and Other Information).

pub struct HNarrativePaths;

impl HNarrativePaths {
	// H.1 Case narrative
	pub const CASE_NARRATIVE: &'static str = "//hl7:investigationEvent/hl7:text";

	// H.2 Reporter comments (author code 3)
	pub const REPORTER_COMMENTS: &'static str =
		"//hl7:component1//hl7:observationEvent[hl7:author/hl7:assignedEntity/hl7:code[@code='3']]/hl7:value";

	// H.4 Sender comments (author code 1)
	pub const SENDER_COMMENTS: &'static str =
		"//hl7:component1//hl7:observationEvent[hl7:author/hl7:assignedEntity/hl7:code[@code='1']]/hl7:value";
}
