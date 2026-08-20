use super::*;

pub(super) fn escape_pdf_text(value: &str) -> String {
	value
		.chars()
		.flat_map(|ch| match ch {
			'(' => "\\(".chars().collect::<Vec<_>>(),
			')' => "\\)".chars().collect::<Vec<_>>(),
			'\\' => "\\\\".chars().collect::<Vec<_>>(),
			ch if ch.is_ascii_control() => " ".chars().collect::<Vec<_>>(),
			_ => vec![ch],
		})
		.collect()
}

pub(super) fn encode_pdf_unicode_text(value: &str) -> String {
	value
		.chars()
		.map(|ch| {
			let codepoint = ch as u32;
			if codepoint <= 0xFFFF {
				format!("{codepoint:04X}")
			} else {
				format!("{codepoint:08X}")
			}
		})
		.collect()
}

pub(super) fn date_text(value: Option<Date>) -> String {
	value.map(|value| value.to_string()).unwrap_or_default()
}

pub(super) fn ts_text(value: Option<&str>) -> String {
	value.unwrap_or_default().to_string()
}

pub(super) fn e2b_datetime_date_text(value: Option<&str>) -> String {
	value
		.and_then(lib_core::serde::flex_date::e2b_datetime_date)
		.map(|value| value.to_string())
		.unwrap_or_default()
}

pub(super) fn decimal_text(value: Option<Decimal>) -> String {
	value
		.map(|value| value.normalize().to_string())
		.unwrap_or_default()
}

pub(super) fn age_unit_text(value: Option<&str>) -> &'static str {
	match value.unwrap_or_default() {
		"a" => "years",
		"mo" => "months",
		"wk" => "weeks",
		"d" => "days",
		"h" => "hours",
		_ => "",
	}
}

pub(super) fn duration_unit_text(value: Option<&str>) -> &'static str {
	match value.unwrap_or_default() {
		"a" => "years",
		"mo" => "months",
		"wk" => "weeks",
		"d" => "days",
		"h" => "hours",
		"min" => "minutes",
		_ => "",
	}
}

pub(super) fn sex_text(value: Option<&str>) -> &'static str {
	match value.unwrap_or_default() {
		"1" => "Male",
		"2" => "Female",
		"0" => "Unknown",
		_ => "",
	}
}

pub(super) fn yes_no_na(value: Option<&str>) -> &'static str {
	match value.unwrap_or_default() {
		"1" => "Yes",
		"2" => "No",
		"3" => "NA",
		_ => "",
	}
}

pub(super) fn yes_no_unknown(value: Option<&str>) -> &'static str {
	match value.unwrap_or_default() {
		"1" => "Yes",
		"2" => "No",
		"3" => "Unknown",
		_ => "",
	}
}

pub(super) fn report_type_text(value: Option<&str>) -> &'static str {
	match value.unwrap_or_default() {
		"1" => "Spontaneous report",
		"2" => "Report from study",
		"3" => "Other",
		"4" => "Not available",
		_ => "",
	}
}

pub(super) fn reaction_outcome_text(value: Option<&str>) -> String {
	match value.unwrap_or_default() {
		"1" => "Recovered/resolved",
		"2" => "Recovering/resolving",
		"3" => "Not recovered/not resolved",
		"4" => "Recovered/resolved with sequelae",
		"5" => "Fatal",
		"6" => "Unknown",
		value => value,
	}
	.to_string()
}

pub(super) fn drug_action_text(value: Option<&str>) -> String {
	match value.unwrap_or_default() {
		"1" => "Drug withdrawn",
		"2" => "Dose reduced",
		"3" => "Dose increased",
		"4" => "Dose not changed",
		"5" => "Unknown",
		"6" => "Not applicable",
		value => value,
	}
	.to_string()
}

pub(super) fn rechallenge_action_text(value: Option<&str>) -> &'static str {
	match value.unwrap_or_default() {
		"1" => "Drug readministered",
		"2" => "Drug not readministered",
		"3" => "Unknown",
		"4" => "Not applicable",
		_ => "",
	}
}

pub(super) fn join_present(values: &[Option<String>], separator: &str) -> String {
	values
		.iter()
		.filter_map(|value| value.as_deref())
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.collect::<Vec<_>>()
		.join(separator)
}

pub(super) fn patient_age(patient: Option<&PatientInformation>) -> String {
	let Some(patient) = patient else {
		return String::new();
	};
	let value = decimal_text(patient.age_at_time_of_onset);
	if value.is_empty() {
		return String::new();
	}
	let unit = age_unit_text(patient.age_unit.as_deref());
	if unit.is_empty() {
		value
	} else {
		format!("{value} {unit}")
	}
}

pub(super) fn reaction_dates(reaction: Option<&Reaction>) -> String {
	let Some(reaction) = reaction else {
		return String::new();
	};
	let start = ts_text(reaction.start_date.as_deref());
	let end = ts_text(reaction.end_date.as_deref());
	match (start.is_empty(), end.is_empty()) {
		(false, false) => format!("{start} to {end}"),
		(false, true) => start,
		(true, false) => end,
		(true, true) => String::new(),
	}
}

pub(super) fn dosage_therapy_dates(dosage: Option<&DosageInformation>) -> String {
	let Some(dosage) = dosage else {
		return String::new();
	};
	let start = date_text(dosage.first_administration_date);
	let end = date_text(dosage.last_administration_date);
	match (start.is_empty(), end.is_empty()) {
		(false, false) => format!("{start} to {end}"),
		(false, true) => start,
		(true, false) => end,
		(true, true) => String::new(),
	}
}

pub(super) fn dosage_duration(dosage: Option<&DosageInformation>) -> String {
	let Some(dosage) = dosage else {
		return String::new();
	};
	let value = decimal_text(dosage.duration_value);
	if value.is_empty() {
		return String::new();
	}
	let unit = duration_unit_text(dosage.duration_unit.as_deref());
	if unit.is_empty() {
		value
	} else {
		format!("{value} {unit}")
	}
}

pub(super) fn drug_name(drug: Option<&DrugInformation>) -> String {
	let Some(drug) = drug else {
		return String::new();
	};
	drug.medicinal_product.clone()
}

pub(super) fn reporter_name(source: Option<&PrimarySource>) -> String {
	let Some(source) = source else {
		return String::new();
	};
	join_present(
		&[
			source.reporter_title.clone(),
			source.reporter_given_name.clone(),
			source.reporter_middle_name.clone(),
			source.reporter_family_name.clone(),
		],
		" ",
	)
}

pub(super) fn sender_address(sender: Option<&SenderInformation>) -> String {
	let Some(sender) = sender else {
		return String::new();
	};
	join_present(
		&[
			sender.organization_name.clone(),
			sender.department.clone(),
			sender.street_address.clone(),
			sender.city.clone(),
			sender.state.clone(),
			sender.postcode.clone(),
			sender.country_code.clone(),
		],
		", ",
	)
}

pub(super) fn concomitant_drugs_text(data: &CiomsCaseData) -> String {
	data.drugs
		.iter()
		.filter(|drug| drug.drug_characterization != "1")
		.map(|drug| {
			let dosage =
				data.dosages
					.iter()
					.filter(|dosage| dosage.drug_id == drug.id)
					.map(|dosage| {
						join_present(
							&[
								dosage.dosage_text.clone(),
								dosage.dose_value.map(|value| {
									format!("Dose: {}", decimal_text(Some(value)))
								}),
								dosage.dose_unit.clone(),
								dosage
									.route_of_administration
									.clone()
									.map(|value| format!("Route: {value}")),
								(!dosage_therapy_dates(Some(dosage)).is_empty())
									.then(|| {
										format!(
											"Dates: {}",
											dosage_therapy_dates(Some(dosage))
										)
									}),
							],
							" | ",
						)
					})
					.filter(|value| !value.is_empty())
					.collect::<Vec<_>>()
					.join("; ");
			let indications = data
				.indications
				.iter()
				.filter(|indication| indication.drug_id == drug.id)
				.filter_map(|indication| indication.indication_text.clone())
				.filter(|value| !value.trim().is_empty())
				.collect::<Vec<_>>()
				.join(", ");
			join_present(
				&[
					Some(drug.medicinal_product.clone()),
					(!dosage.is_empty())
						.then(|| format!("Dose/route/dates: {dosage}")),
					(!indications.is_empty())
						.then(|| format!("Indication: {indications}")),
					drug.action_taken.as_deref().map(|value| {
						format!("Action: {}", drug_action_text(Some(value)))
					}),
				],
				" | ",
			)
		})
		.collect::<Vec<_>>()
		.join("; ")
}
