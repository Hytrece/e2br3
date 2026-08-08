// MFDS mapping for Section G (Drug/Biologic).

pub struct GMfdsDrugPaths;

impl GMfdsDrugPaths {
	pub const MPID: &'static str =
		"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code[@codeSystem='2.16.840.1.113883.3.989.5.1.10.2.1']/@code";
	pub const MPID_VERSION: &'static str =
		"hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code[@codeSystem='2.16.840.1.113883.3.989.5.1.10.2.1']/@codeSystemVersion";
	pub const SUBSTANCE_ID: &'static str =
		"hl7:ingredientSubstance/hl7:code[@codeSystem='2.16.840.1.113883.3.989.5.1.10.2.2']/@code";
	pub const SUBSTANCE_VERSION: &'static str =
		"hl7:ingredientSubstance/hl7:code[@codeSystem='2.16.840.1.113883.3.989.5.1.10.2.2']/@codeSystemVersion";

	pub const KR_FIELDS: &'static [&'static str] = &[
		"G.k.2.1.KR.1a",
		"G.k.2.1.KR.1b",
		"G.k.2.3.r.1.KR.1a",
		"G.k.2.3.r.1.KR.1b",
		"G.k.9.i.2.r.2.KR.1",
		"G.k.9.i.2.r.3.KR.1",
		"G.k.9.i.2.r.3.KR.2",
	];

	// Note: G.k.9.i.2.r.3.KR.2 is recognized as an MFDS field id, but the
	// canonical XML source path is not yet defined in local mappings/fixtures, so
	// import currently leaves it unsupported on purpose.
}
