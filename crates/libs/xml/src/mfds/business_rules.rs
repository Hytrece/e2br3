// MFDS regional business-rule scaffolding.
//
// Goal: keep regional rule definitions separate from FDA and ICH core rules.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSeverity {
	Reject,
	Warning,
	Info,
}

#[derive(Debug, Clone, Copy)]
pub struct MfdsRuleSpec {
	pub id: &'static str,
	pub title: &'static str,
	pub severity: RuleSeverity,
	pub note: &'static str,
}

// Initial MFDS rule catalog distilled from implementation references.
// Replace notes with official MFDS guide citations when parsed from source files.
pub const MFDS_RULES: &[MfdsRuleSpec] = &[
	MfdsRuleSpec {
		id: "MFDS.C.5.4.KR.1.REQ",
		title: "C.5.4.KR.1 required when C.5.4 is Other studies",
		severity: RuleSeverity::Reject,
		note: "If C.5.4=3 (Other studies), C.5.4.KR.1 must be populated.",
	},
	MfdsRuleSpec {
		id: "MFDS.C.2.R.4.KR.1.COND",
		title: "C.2.r.4.KR.1 conditional on reporter qualification",
		severity: RuleSeverity::Reject,
		note: "Required when C.2.r.4 indicates Other health professional; otherwise not accepted.",
	},
	MfdsRuleSpec {
		id: "MFDS.C.3.1.KR.1.COND",
		title: "C.3.1.KR.1 conditional for sender type",
		severity: RuleSeverity::Reject,
		note: "Expected when C.3.1 sender type is health professional.",
	},
	MfdsRuleSpec {
		id: "MFDS.KR.DOMESTIC.PRODUCTCODE.COND",
		title: "KR product/ingredient domestic code conditional",
		severity: RuleSeverity::Reject,
		note: "For domestic post-marketed cases, KR product and ingredient codes become conditionally required.",
	},
	MfdsRuleSpec {
		id: "MFDS.KR.FOREIGN.WHOMPID.COND",
		title: "KR product/ingredient WHO-MPID conditional",
		severity: RuleSeverity::Reject,
		note: "For foreign-use products, WHO MPID pathway applies for KR product/ingredient fields.",
	},
	MfdsRuleSpec {
		id: "MFDS.G.K.9.I.2.R.3.KR.SOURCEMETHOD",
		title: "KR causality method/result values constrained by source",
		severity: RuleSeverity::Reject,
		note: "Allowed values for KR.1/KR.2 depend on source-of-assessment categories.",
	},
];
