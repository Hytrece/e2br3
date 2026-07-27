// MFDS mapping for Section C (Safety Report, Sender/Reporter).
//
// Keep ICH core paths in shared mappings and place KR-only paths here.

pub struct CMfdsSafetyReportPaths;

impl CMfdsSafetyReportPaths {
	// Regional field IDs implemented in this section.
	pub const KR_FIELDS: &'static [&'static str] =
		&["C.2.r.4.KR.1", "C.3.1.KR.1", "C.5.4.KR.1"];
}
