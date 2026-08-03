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
	Base64,
}
