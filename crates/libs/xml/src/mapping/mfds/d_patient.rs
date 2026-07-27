// MFDS mapping for Section D (Patient).
//
// Regional KR fields should be declared here; ICH core remains shared.

pub struct DMfdsPatientPaths;

impl DMfdsPatientPaths {
	pub const KR_FIELDS: &'static [&'static str] = &[
		"D.8.r.1.KR.1a",
		"D.8.r.1.KR.1b",
		"D.10.8.r.1.KR.1a",
		"D.10.8.r.1.KR.1b",
	];
}
