mod helpers;

pub mod generated;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputValue<'a> {
	Missing,
	String(&'a str),
	Boolean(bool),
	Number(&'a serde_json::Number),
	InvalidType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldInput<'a> {
	pub value: InputValue<'a>,
	pub null_flavor: Option<&'a str>,
}

impl<'a> FieldInput<'a> {
	pub const fn new(value: InputValue<'a>, null_flavor: Option<&'a str>) -> Self {
		Self { value, null_flavor }
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputIssue {
	pub code: &'static str,
	pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum NumericShape {
	Decimal,
	DottedVersion,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FormatName {
	E2bDatetime,
}

#[cfg(test)]
mod tests {
	use super::{generated, FieldInput, InputValue};

	#[test]
	fn migrated_business_rules_do_not_reject_input() {
		assert!(generated::c::c_1_6_1_r_2(FieldInput::new(
			InputValue::String("not-base64"),
			None,
		))
		.is_empty());
		assert!(generated::c::c_4_r_2(FieldInput::new(
			InputValue::String("not-base64"),
			None,
		))
		.is_empty());
		assert!(generated::d::d_6(FieldInput::new(
			InputValue::Missing,
			Some("NASK"),
		))
		.is_empty());
		assert!(generated::e::e_i_2_1a(FieldInput::new(
			InputValue::String("15"),
			None,
		))
		.is_empty());
		assert!(generated::d::d_7_1_r_1a(FieldInput::new(
			InputValue::String("15"),
			None,
		))
		.is_empty());
		assert!(generated::d::d_10_7_1_r_1a(FieldInput::new(
			InputValue::String("15"),
			None,
		))
		.is_empty());
		assert!(generated::g::g_k_7_r_2a(FieldInput::new(
			InputValue::String("15"),
			None,
		))
		.is_empty());
		assert!(generated::f::f_r_2_2a(FieldInput::new(
			InputValue::String("15"),
			None,
		))
		.is_empty());
		assert!(generated::h::h_3_r_1a(FieldInput::new(
			InputValue::String("15"),
			None,
		))
		.is_empty());
	}
}
