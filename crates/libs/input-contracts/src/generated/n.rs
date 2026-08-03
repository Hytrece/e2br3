// Generated once from registry/dictionary/*.json; explicit field functions are maintained here.

/// ICH.N.1.1.LENGTH.MAX
/// ICH.N.1.1.ALLOWED.VALUE
pub fn n_1_1(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::max_length(&mut issues, "ICH.N.1.1.LENGTH.MAX", input.value, 2);
	crate::helpers::allowed_values(
		&mut issues,
		"ICH.N.1.1.ALLOWED.VALUE",
		input.value,
		&["1"],
	);
	issues
}

/// ICH.N.1.2.LENGTH.MAX
pub fn n_1_2(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::max_length(
		&mut issues,
		"ICH.N.1.2.LENGTH.MAX",
		input.value,
		100,
	);
	issues
}

/// ICH.N.1.3.LENGTH.MAX
pub fn n_1_3(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::max_length(&mut issues, "ICH.N.1.3.LENGTH.MAX", input.value, 60);
	issues
}

/// ICH.N.1.4.LENGTH.MAX
pub fn n_1_4(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::max_length(&mut issues, "ICH.N.1.4.LENGTH.MAX", input.value, 60);
	issues
}

/// ICH.N.1.5.ALLOWED.VALUE
pub fn n_1_5(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::format(
		&mut issues,
		"ICH.N.1.5.ALLOWED.VALUE",
		input.value,
		crate::FormatName::E2bDatetime,
	);
	issues
}

/// ICH.N.2.r.1.LENGTH.MAX
pub fn n_2_r_1(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::max_length(
		&mut issues,
		"ICH.N.2.r.1.LENGTH.MAX",
		input.value,
		100,
	);
	issues
}

/// ICH.N.2.r.2.LENGTH.MAX
pub fn n_2_r_2(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::max_length(
		&mut issues,
		"ICH.N.2.r.2.LENGTH.MAX",
		input.value,
		60,
	);
	issues
}

/// ICH.N.2.r.3.LENGTH.MAX
pub fn n_2_r_3(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::max_length(
		&mut issues,
		"ICH.N.2.r.3.LENGTH.MAX",
		input.value,
		60,
	);
	issues
}

/// ICH.N.2.r.4.ALLOWED.VALUE
pub fn n_2_r_4(input: crate::FieldInput<'_>) -> Vec<crate::InputIssue> {
	let mut issues = Vec::new();
	crate::helpers::format(
		&mut issues,
		"ICH.N.2.r.4.ALLOWED.VALUE",
		input.value,
		crate::FormatName::E2bDatetime,
	);
	issues
}
