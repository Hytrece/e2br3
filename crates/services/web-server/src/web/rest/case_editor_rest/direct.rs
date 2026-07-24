use super::common::*;
use lib_core::model::safety_report::PrimarySource;
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaseEditorRpPrimarySourceDto {
	id: Uuid,
	sequence_number: i32,
	reporter_title: Option<String>,
	reporter_title_null_flavor: Option<String>,
	reporter_given_name: Option<String>,
	reporter_given_name_null_flavor: Option<String>,
	reporter_middle_name: Option<String>,
	reporter_middle_name_null_flavor: Option<String>,
	reporter_family_name: Option<String>,
	reporter_family_name_null_flavor: Option<String>,
	#[serde(rename = "reporterOrganization")]
	organization: Option<String>,
	#[serde(rename = "reporterOrganizationNullFlavor")]
	organization_null_flavor: Option<String>,
	#[serde(rename = "reporterDepartment")]
	department: Option<String>,
	#[serde(rename = "reporterDepartmentNullFlavor")]
	department_null_flavor: Option<String>,
	#[serde(rename = "reporterStreet")]
	street: Option<String>,
	#[serde(rename = "reporterStreetNullFlavor")]
	street_null_flavor: Option<String>,
	#[serde(rename = "reporterCity")]
	city: Option<String>,
	#[serde(rename = "reporterCityNullFlavor")]
	city_null_flavor: Option<String>,
	#[serde(rename = "reporterState")]
	state: Option<String>,
	#[serde(rename = "reporterStateNullFlavor")]
	state_null_flavor: Option<String>,
	#[serde(rename = "reporterPostcode")]
	postcode: Option<String>,
	#[serde(rename = "reporterPostcodeNullFlavor")]
	postcode_null_flavor: Option<String>,
	#[serde(rename = "reporterTelephone")]
	telephone: Option<String>,
	#[serde(rename = "reporterTelephoneNullFlavor")]
	telephone_null_flavor: Option<String>,
	#[serde(rename = "reporterCountry")]
	country_code: Option<String>,
	#[serde(rename = "reporterCountryNullFlavor")]
	country_code_null_flavor: Option<String>,
	#[serde(rename = "reporterEmail")]
	email: Option<String>,
	#[serde(rename = "reporterEmailNullFlavor")]
	email_null_flavor: Option<String>,
	qualification: Option<String>,
	qualification_null_flavor: Option<String>,
	qualification_kr1: Option<String>,
	#[serde(rename = "primarySourceForRegulatoryPurposes")]
	primary_source_regulatory: Option<String>,
}

impl From<PrimarySource> for CaseEditorRpPrimarySourceDto {
	fn from(source: PrimarySource) -> Self {
		Self {
			id: source.id,
			sequence_number: source.sequence_number,
			reporter_title: source.reporter_title,
			reporter_title_null_flavor: source.reporter_title_null_flavor,
			reporter_given_name: source.reporter_given_name,
			reporter_given_name_null_flavor: source.reporter_given_name_null_flavor,
			reporter_middle_name: source.reporter_middle_name,
			reporter_middle_name_null_flavor: source
				.reporter_middle_name_null_flavor,
			reporter_family_name: source.reporter_family_name,
			reporter_family_name_null_flavor: source
				.reporter_family_name_null_flavor,
			organization: source.organization,
			organization_null_flavor: source.organization_null_flavor,
			department: source.department,
			department_null_flavor: source.department_null_flavor,
			street: source.street,
			street_null_flavor: source.street_null_flavor,
			city: source.city,
			city_null_flavor: source.city_null_flavor,
			state: source.state,
			state_null_flavor: source.state_null_flavor,
			postcode: source.postcode,
			postcode_null_flavor: source.postcode_null_flavor,
			telephone: source.telephone,
			telephone_null_flavor: source.telephone_null_flavor,
			country_code: source.country_code,
			country_code_null_flavor: source.country_code_null_flavor,
			email: source.email,
			email_null_flavor: source.email_null_flavor,
			qualification: source.qualification,
			qualification_null_flavor: source.qualification_null_flavor,
			qualification_kr1: source.qualification_kr1,
			primary_source_regulatory: source.primary_source_regulatory,
		}
	}
}

fn ci_date(value: Option<sqlx::types::time::Date>) -> Option<String> {
	value.map(|date| {
		format!(
			"{:04}{:02}{:02}",
			date.year(),
			u8::from(date.month()),
			date.day()
		)
	})
}

async fn load_editor_ci_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Value> {
	let case = CaseBmc::get(ctx, mm, case_id).await?;
	let case_fields = CaseEditorCiCaseDto {
		report_year: case.report_year,
		fda_report_type: case.fda_report_type,
		mfds_report_type: case.mfds_report_type,
	};
	let safety_report_identification =
		match SafetyReportIdentificationBmc::get_by_case(ctx, mm, case_id).await {
			Ok(entity) => Some(CaseEditorCiSafetyReportDto {
				id: entity.id,
				safety_report_id: entity.safety_report_id,
				transmission_date: entity.transmission_date,
				report_type: entity.report_type,
				date_first_received_from_source: ci_date(
					entity.date_first_received_from_source,
				),
				date_of_most_recent_information: ci_date(
					entity.date_of_most_recent_information,
				),
				fulfil_expedited_criteria: entity.fulfil_expedited_criteria,
				fulfil_expedited_criteria_null_flavor: entity
					.fulfil_expedited_criteria_null_flavor,
				local_criteria_report_type: entity.local_criteria_report_type,
				combination_product_report_indicator: entity
					.combination_product_report_indicator,
				combination_product_report_indicator_null_flavor: entity
					.combination_product_report_indicator_null_flavor,
				worldwide_unique_id: entity.worldwide_unique_id,
				first_sender_type: entity.first_sender_type,
				additional_documents_available: entity
					.additional_documents_available,
				other_case_identifiers_exist: entity.other_case_identifiers_exist,
				other_case_identifiers_exist_null_flavor: entity
					.other_case_identifiers_exist_null_flavor,
				nullification_amendment_code: entity.nullification_code,
				nullification_reason: entity.nullification_reason,
			}),
			Err(lib_core::model::Error::EntityUuidNotFound { .. }) => None,
			Err(err) => return Err(err.into()),
		};
	let other_case_identifiers = OtherCaseIdentifierBmc::list(
		ctx,
		mm,
		Some(vec![OtherCaseIdentifierFilter {
			case_id: Some(uuid_eq(case_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let linked_reports = LinkedReportNumberBmc::list(
		ctx,
		mm,
		Some(vec![LinkedReportNumberFilter {
			case_id: Some(uuid_eq(case_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let documents_held_by_sender = DocumentsHeldBySenderBmc::list(
		ctx,
		mm,
		Some(vec![DocumentsHeldBySenderFilter {
			case_id: Some(uuid_eq(case_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let source_documents = SourceDocumentBmc::list(
		ctx,
		mm,
		Some(vec![SourceDocumentFilter {
			case_id: Some(uuid_eq(case_id)),
		}]),
		Some(ListOptions::default()),
	)
	.await?;

	Ok(json!(CaseEditorCiRowsDto {
		case: case_fields,
		safety_report_identification,
		other_case_identifiers: other_case_identifiers
			.into_iter()
			.map(|row| CaseEditorCiOtherIdentifierDto {
				id: row.id,
				source: row.source_of_identifier,
				case_identifier: row.case_identifier,
				sequence_number: row.sequence_number,
				deleted: row.deleted,
			})
			.collect(),
		linked_reports: linked_reports
			.into_iter()
			.map(|row| CaseEditorCiLinkedReportDto {
				id: row.id,
				linked_report_number: row.linked_report_number,
				sequence_number: row.sequence_number,
				deleted: row.deleted,
			})
			.collect(),
		documents_held_by_sender: documents_held_by_sender
			.into_iter()
			.map(|row| CaseEditorCiDocumentDto {
				id: row.id,
				document_description: row.title,
				included_document: row.document_base64,
				media_type: row.media_type,
				representation: row.representation,
				compression: row.compression,
				sequence_number: row.sequence_number,
				deleted: row.deleted,
			})
			.collect(),
		source_documents: source_documents
			.into_iter()
			.map(|row| CaseEditorCiSourceDocumentDto {
				id: row.id,
				source_document_name: row.source_document_name,
				source_document_base64: row.source_document_base64,
				source_document_media_type: row.source_document_media_type,
				sequence_number: row.sequence_number,
			})
			.collect(),
	}))
}

pub async fn get_editor_ci(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, SAFETY_REPORT_READ)?;
	require_permission(&ctx, MESSAGE_HEADER_READ)?;
	require_permission(&ctx, RECEIVER_READ)?;
	require_permission(&ctx, CASE_IDENTIFIER_LIST)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	Ok(direct_section_response(
		case_id,
		load_editor_ci_data(&ctx, &mm, case_id).await?,
	))
}

/// GET /api/cases/{case_id}/editor/pages/CI
pub async fn get_editor_ci_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Query(query): Query<CaseEditorPageProjectionQuery>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, SAFETY_REPORT_READ)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	let projection = direct_page_projection_response(
		&ctx,
		&mm,
		case_id,
		"CI",
		query_authorities_csv(&query)?,
		load_editor_ci_data(&ctx, &mm, case_id).await?,
	)
	.await?;
	Ok((axum::http::StatusCode::OK, Json(projection)))
}

fn patch_string_value(
	field_name: &str,
	patch: &CaseEditorFieldPatch,
) -> Result<PatchValue<String>> {
	let Some(value) = patch.value.as_ref() else {
		return Ok(PatchValue::Missing);
	};
	if value.is_null() {
		return Ok(PatchValue::Null);
	}
	let Some(value) = value.as_str() else {
		return Err(Error::BadRequest {
			message: format!("{field_name} must be a string or null"),
		});
	};
	Ok(PatchValue::Value(value.trim().to_string()))
}

fn patch_bool_value(
	field_name: &str,
	patch: &CaseEditorFieldPatch,
) -> Result<PatchValue<bool>> {
	let Some(value) = patch.value.as_ref() else {
		return Ok(PatchValue::Missing);
	};
	if value.is_null() {
		return Ok(PatchValue::Null);
	}
	let Some(value) = value.as_bool() else {
		return Err(Error::BadRequest {
			message: format!("{field_name} must be a boolean or null"),
		});
	};
	Ok(PatchValue::Value(value))
}

fn patch_optional_string_value(
	field_name: &str,
	patch: &CaseEditorFieldPatch,
) -> Result<Option<String>> {
	let Some(value) = patch.value.as_ref() else {
		return Ok(None);
	};
	if value.is_null() {
		return Ok(None);
	}
	let Some(value) = value.as_str() else {
		return Err(Error::BadRequest {
			message: format!("{field_name} must be a string or null"),
		});
	};
	Ok(Some(value.trim().to_string()))
}

fn patch_optional_bool_value(
	field_name: &str,
	patch: &CaseEditorFieldPatch,
) -> Result<Option<bool>> {
	let Some(value) = patch.value.as_ref() else {
		return Ok(None);
	};
	if value.is_null() {
		return Ok(None);
	}
	let Some(value) = value.as_bool() else {
		return Err(Error::BadRequest {
			message: format!("{field_name} must be a boolean or null"),
		});
	};
	Ok(Some(value))
}

#[derive(Deserialize)]
struct CiDatePatchValue {
	#[serde(
		default,
		deserialize_with = "lib_core::serde::flex_date::deserialize_option_date"
	)]
	value: Option<sqlx::types::time::Date>,
}

fn patch_date_value(
	field_name: &str,
	patch: &CaseEditorFieldPatch,
) -> Result<Option<sqlx::types::time::Date>> {
	let value = patch.value.clone().unwrap_or(Value::Null);
	serde_json::from_value::<CiDatePatchValue>(json!({ "value": value }))
		.map(|value| value.value)
		.map_err(|err| Error::BadRequest {
			message: format!("{field_name} must be an E2B date or null: {err}"),
		})
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CiCaseRowPatch {
	report_year: Option<String>,
	fda_report_type: Option<String>,
	mfds_report_type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CiDocumentRowPatch {
	#[serde(default)]
	id: Option<Uuid>,
	#[serde(default)]
	document_description: Option<String>,
	#[serde(default)]
	included_document: Option<String>,
	#[serde(default)]
	media_type: Option<String>,
	#[serde(default)]
	representation: Option<String>,
	#[serde(default)]
	compression: Option<String>,
	#[serde(default)]
	sequence_number: Option<i32>,
	#[serde(default)]
	deleted: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CiOtherIdentifierRowPatch {
	#[serde(default)]
	id: Option<Uuid>,
	#[serde(default)]
	source: Option<String>,
	#[serde(default)]
	case_identifier: Option<String>,
	#[serde(default)]
	sequence_number: Option<i32>,
	#[serde(default)]
	deleted: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CiLinkedReportRowPatch {
	#[serde(default)]
	id: Option<Uuid>,
	#[serde(default)]
	linked_report_number: Option<String>,
	#[serde(default)]
	sequence_number: Option<i32>,
	#[serde(default)]
	deleted: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CiSourceDocumentRowPatch {
	#[serde(default)]
	id: Option<Uuid>,
	#[serde(default)]
	source_document_name: Option<String>,
	#[serde(default)]
	source_document_base64: Option<String>,
	#[serde(default)]
	source_document_media_type: Option<String>,
	#[serde(default)]
	sequence_number: Option<i32>,
}

fn ci_row_error(owner: &str, err: serde_json::Error) -> Error {
	Error::BadRequest {
		message: format!("CI.{owner} has an invalid row payload: {err}"),
	}
}

async fn apply_ci_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	if rows.contains_key("messageHeader")
		|| rows.contains_key("safetyReportIdentification")
	{
		return Err(Error::BadRequest {
			message: "CI.messageHeader and CI.safetyReportIdentification row patches are not supported; use changes".to_string(),
		});
	}
	reject_unknown_row_keys(
		"CI",
		rows,
		&[
			"case",
			"messageHeader",
			"safetyReportIdentification",
			"documentsHeldBySender",
			"otherCaseIdentifiers",
			"linkedReports",
			"sourceDocuments",
		],
	)?;

	let portable_row = rows
		.iter()
		.map(|(key, value)| (key.clone(), value.clone()))
		.collect::<Map<String, Value>>();
	validate_row_payload("CI", "CI", &portable_row, None)?;

	if let Some(value) = rows.get("case") {
		let patch = serde_json::from_value::<CiCaseRowPatch>(value.clone())
			.map_err(|err| ci_row_error("case", err))?;
		CaseBmc::update(
			ctx,
			mm,
			case_id,
			CaseForUpdate {
				report_year: patch.report_year,
				fda_report_type: patch.fda_report_type,
				mfds_report_type: patch.mfds_report_type,
				dirty_c: Some(true),
				..Default::default()
			},
		)
		.await?;
	}

	if let Some(value) = rows.get("documentsHeldBySender") {
		let patches =
			serde_json::from_value::<Vec<CiDocumentRowPatch>>(value.clone())
				.map_err(|err| ci_row_error("documentsHeldBySender", err))?;
		for (index, patch) in patches.into_iter().enumerate() {
			let sequence_number = patch
				.sequence_number
				.unwrap_or_else(|| i32::try_from(index + 1).unwrap_or(i32::MAX));
			if let Some(id) = patch.id {
				let current = DocumentsHeldBySenderBmc::get(ctx, mm, id).await?;
				if current.case_id != case_id {
					return Err(Error::BadRequest {
						message: format!("CI.documentsHeldBySender row '{id}' belongs to another case"),
					});
				}
				if patch.deleted == Some(true) {
					DocumentsHeldBySenderBmc::delete(ctx, mm, id).await?;
					continue;
				}
				if patch.deleted == Some(false) && current.deleted {
					DocumentsHeldBySenderBmc::restore(ctx, mm, id).await?;
				}
				DocumentsHeldBySenderBmc::update(
					ctx,
					mm,
					id,
					DocumentsHeldBySenderForUpdate {
						title: patch.document_description,
						document_base64: patch.included_document,
						media_type: patch.media_type,
						representation: patch.representation,
						compression: patch.compression,
						sequence_number: patch.sequence_number,
					},
				)
				.await?;
			} else {
				if patch.deleted == Some(true) {
					return Err(Error::BadRequest {
						message:
							"a new CI.documentsHeldBySender row cannot be deleted"
								.to_string(),
					});
				}
				DocumentsHeldBySenderBmc::create(
					ctx,
					mm,
					DocumentsHeldBySenderForCreate {
						case_id,
						title: patch.document_description,
						document_base64: patch.included_document,
						media_type: patch.media_type,
						representation: patch.representation,
						compression: patch.compression,
						sequence_number,
					},
				)
				.await?;
			}
		}
	}

	if let Some(value) = rows.get("otherCaseIdentifiers") {
		let patches =
			serde_json::from_value::<Vec<CiOtherIdentifierRowPatch>>(value.clone())
				.map_err(|err| ci_row_error("otherCaseIdentifiers", err))?;
		for (index, patch) in patches.into_iter().enumerate() {
			if let Some(id) = patch.id {
				let current = OtherCaseIdentifierBmc::get(ctx, mm, id).await?;
				if current.case_id != case_id {
					return Err(Error::BadRequest {
						message: format!("CI.otherCaseIdentifiers row '{id}' belongs to another case"),
					});
				}
				if patch.deleted == Some(true) {
					OtherCaseIdentifierBmc::delete(ctx, mm, id).await?;
					continue;
				}
				if patch.deleted == Some(false) && current.deleted {
					OtherCaseIdentifierBmc::restore(ctx, mm, id).await?;
				}
				OtherCaseIdentifierBmc::update(
					ctx,
					mm,
					id,
					OtherCaseIdentifierForUpdate {
						source_of_identifier: patch.source,
						case_identifier: patch.case_identifier,
					},
				)
				.await?;
			} else {
				let source = patch.source.ok_or_else(|| Error::BadRequest {
					message: "CI.otherCaseIdentifiers[].source is required"
						.to_string(),
				})?;
				let case_identifier =
					patch.case_identifier.ok_or_else(|| Error::BadRequest {
						message:
							"CI.otherCaseIdentifiers[].caseIdentifier is required"
								.to_string(),
					})?;
				OtherCaseIdentifierBmc::create(
					ctx,
					mm,
					OtherCaseIdentifierForCreate {
						case_id,
						sequence_number: patch.sequence_number.unwrap_or_else(
							|| i32::try_from(index + 1).unwrap_or(i32::MAX),
						),
						source_of_identifier: source,
						case_identifier,
					},
				)
				.await?;
			}
		}
	}

	if let Some(value) = rows.get("linkedReports") {
		let patches =
			serde_json::from_value::<Vec<CiLinkedReportRowPatch>>(value.clone())
				.map_err(|err| ci_row_error("linkedReports", err))?;
		for (index, patch) in patches.into_iter().enumerate() {
			if let Some(id) = patch.id {
				let current = LinkedReportNumberBmc::get(ctx, mm, id).await?;
				if current.case_id != case_id {
					return Err(Error::BadRequest {
						message: format!(
							"CI.linkedReports row '{id}' belongs to another case"
						),
					});
				}
				if patch.deleted == Some(true) {
					LinkedReportNumberBmc::delete(ctx, mm, id).await?;
					continue;
				}
				if patch.deleted == Some(false) && current.deleted {
					LinkedReportNumberBmc::restore(ctx, mm, id).await?;
				}
				LinkedReportNumberBmc::update(
					ctx,
					mm,
					id,
					LinkedReportNumberForUpdate {
						linked_report_number: patch.linked_report_number,
					},
				)
				.await?;
			} else {
				let linked_report_number =
					patch
						.linked_report_number
						.ok_or_else(|| Error::BadRequest {
							message:
								"CI.linkedReports[].linkedReportNumber is required"
									.to_string(),
						})?;
				LinkedReportNumberBmc::create(
					ctx,
					mm,
					LinkedReportNumberForCreate {
						case_id,
						sequence_number: patch.sequence_number.unwrap_or_else(
							|| i32::try_from(index + 1).unwrap_or(i32::MAX),
						),
						linked_report_number,
					},
				)
				.await?;
			}
		}
	}

	if let Some(value) = rows.get("sourceDocuments") {
		let patches =
			serde_json::from_value::<Vec<CiSourceDocumentRowPatch>>(value.clone())
				.map_err(|err| ci_row_error("sourceDocuments", err))?;
		for (index, patch) in patches.into_iter().enumerate() {
			if let Some(id) = patch.id {
				let current = SourceDocumentBmc::get(ctx, mm, id).await?;
				if current.case_id != case_id {
					return Err(Error::BadRequest {
						message: format!(
							"CI.sourceDocuments row '{id}' belongs to another case"
						),
					});
				}
				SourceDocumentBmc::update(
					ctx,
					mm,
					id,
					SourceDocumentForUpdate {
						source_document_name: patch.source_document_name,
						source_document_base64: patch.source_document_base64,
						source_document_media_type: patch.source_document_media_type,
						sequence_number: patch.sequence_number,
					},
				)
				.await?;
			} else {
				SourceDocumentBmc::create(
					ctx,
					mm,
					SourceDocumentForCreate {
						case_id,
						source_document_name: patch.source_document_name,
						source_document_base64: patch.source_document_base64,
						source_document_media_type: patch.source_document_media_type,
						sequence_number: patch.sequence_number.unwrap_or_else(
							|| i32::try_from(index + 1).unwrap_or(i32::MAX),
						),
					},
				)
				.await?;
			}
		}
	}

	Ok(())
}

/// PATCH /api/cases/{case_id}/editor/pages/CI
pub async fn patch_editor_ci_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_UPDATE)?;
	require_permission(&ctx, SAFETY_REPORT_UPDATE)?;
	lib_rest_core::require_case_write_allowed(&ctx, &mm, case_id).await?;
	let requested_authorities =
		validate_request_projection_context(request.authorities.as_deref())?;
	validate_direct_changes("CI", &request.changes)?;

	let mut update = SafetyReportIdentificationForUpdate {
		safety_report_id: None,
		version: None,
		transmission_date: None,
		report_type: PatchValue::Missing,
		date_first_received_from_source: None,
		date_of_most_recent_information: None,
		fulfil_expedited_criteria: PatchValue::Missing,
		fulfil_expedited_criteria_null_flavor: None,
		local_criteria_report_type: PatchValue::Missing,
		combination_product_report_indicator: PatchValue::Missing,
		combination_product_report_indicator_null_flavor: None,
		worldwide_unique_id: None,
		first_sender_type: None,
		additional_documents_available: None,
		other_case_identifiers_exist: None,
		other_case_identifiers_exist_null_flavor: None,
		nullification_code: None,
		nullification_reason: None,
		receiver_organization: None,
	};

	for (field, patch) in &request.changes {
		match field.as_str() {
			"safetyReportId" => {
				update.safety_report_id = patch_optional_string_value(field, patch)?;
			}
			"transmissionDate" => {
				update.transmission_date =
					patch_optional_string_value(field, patch)?;
			}
			"reportType" => {
				update.report_type = patch_string_value(field, patch)?;
			}
			"dateFirstReceivedFromSource" => {
				update.date_first_received_from_source =
					patch_date_value(field, patch)?;
			}
			"dateOfMostRecentInformation" => {
				update.date_of_most_recent_information =
					patch_date_value(field, patch)?;
			}
			"additionalDocumentsAvailable" => {
				update.additional_documents_available =
					patch_optional_bool_value(field, patch)?;
			}
			"fulfilExpeditedCriteria" => {
				update.fulfil_expedited_criteria = patch_bool_value(field, patch)?;
			}
			"fulfilExpeditedCriteriaNullFlavor" => {
				update.fulfil_expedited_criteria_null_flavor =
					patch_optional_string_value(field, patch)?;
			}
			"localCriteriaReportType" => {
				update.local_criteria_report_type =
					patch_string_value(field, patch)?;
			}
			"combinationProductReportIndicator" => {
				update.combination_product_report_indicator =
					patch_string_value(field, patch)?;
			}
			"combinationProductReportIndicatorNullFlavor" => {
				update.combination_product_report_indicator_null_flavor =
					patch_optional_string_value(field, patch)?;
			}
			"worldwideUniqueId" => {
				update.worldwide_unique_id =
					patch_optional_string_value(field, patch)?;
			}
			"firstSenderType" => {
				update.first_sender_type =
					patch_optional_string_value(field, patch)?;
			}
			"otherCaseIdentifiersExist" => {
				update.other_case_identifiers_exist =
					patch_optional_bool_value(field, patch)?;
			}
			"otherCaseIdentifiersExistNullFlavor" => {
				update.other_case_identifiers_exist_null_flavor =
					patch_optional_string_value(field, patch)?;
			}
			"nullificationAmendmentCode" => {
				update.nullification_code =
					patch_optional_string_value(field, patch)?;
			}
			"nullificationReason" => {
				update.nullification_reason =
					patch_optional_string_value(field, patch)?;
			}
			_ => {
				return Err(Error::BadRequest {
					message: format!("unknown CI field '{field}'"),
				});
			}
		}
	}
	if !request.changes.is_empty() {
		SafetyReportIdentificationBmc::update_by_case(&ctx, &mm, case_id, update)
			.await?;
	}
	if !request.rows.is_empty() {
		apply_ci_rows_patch(&ctx, &mm, case_id, &request.rows).await?;
	}
	if !request.changes.is_empty() || !request.rows.is_empty() {
		refresh_editor_validation_cache(
			&ctx,
			&mm,
			case_id,
			requested_authorities.clone(),
		)
		.await?;
	}
	let projection = direct_page_projection_response(
		&ctx,
		&mm,
		case_id,
		"CI",
		requested_authorities,
		load_editor_ci_data(&ctx, &mm, case_id).await?,
	)
	.await?;
	Ok((axum::http::StatusCode::OK, Json(projection)))
}

pub async fn patch_editor_rp_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, case_id, "RP", request).await
}

pub async fn patch_editor_sd_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, case_id, "SD", request).await
}

pub async fn patch_editor_lr_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, case_id, "LR", request).await
}

pub async fn patch_editor_si_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, case_id, "SI", request).await
}

pub async fn patch_editor_dm_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, case_id, "DM", request).await
}

pub async fn patch_editor_nr_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, case_id, "NR", request).await
}

async fn patch_direct_page_projection(
	mm: ModelManager,
	ctx_w: CtxW,
	case_id: Uuid,
	page_id: &'static str,
	request: CaseEditorPagePatchRequest,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_UPDATE)?;
	require_permission(&ctx, SAFETY_REPORT_UPDATE)?;
	lib_rest_core::require_case_write_allowed(&ctx, &mm, case_id).await?;
	let requested_authorities =
		validate_request_projection_context(request.authorities.as_deref())?;
	validate_direct_changes(page_id, &request.changes)?;
	validate_direct_rows(page_id, &request.rows)?;

	if !request.changes.is_empty() {
		apply_direct_page_changes_patch(
			&ctx,
			&mm,
			case_id,
			page_id,
			&request.changes,
		)
		.await?;
	}

	if !request.rows.is_empty() {
		apply_direct_page_rows_patch(&ctx, &mm, case_id, page_id, &request.rows)
			.await?;
	}

	if !request.changes.is_empty() || !request.rows.is_empty() {
		refresh_editor_validation_cache(
			&ctx,
			&mm,
			case_id,
			requested_authorities.clone(),
		)
		.await?;
	}

	let data = match page_id {
		"RP" => load_editor_rp_data(&ctx, &mm, case_id).await?,
		"SD" => load_editor_sd_data(&ctx, &mm, case_id).await?,
		"LR" => load_editor_lr_data(&ctx, &mm, case_id).await?,
		"SI" => load_editor_si_data(&ctx, &mm, case_id).await?,
		"DM" => load_editor_dm_data(&ctx, &mm, case_id).await?,
		"NR" => load_editor_nr_data(&ctx, &mm, case_id).await?,
		_ => {
			return Err(Error::BadRequest {
				message: format!("unsupported direct page '{page_id}'"),
			})
		}
	};
	let projection = direct_page_projection_response(
		&ctx,
		&mm,
		case_id,
		page_id,
		requested_authorities,
		data,
	)
	.await?;
	Ok((axum::http::StatusCode::OK, Json(projection)))
}

async fn apply_direct_page_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	match page_id {
		"RP" => apply_rp_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		"SD" => apply_sd_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		"LR" => apply_lr_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		"SI" => apply_si_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		"DM" => apply_dm_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		"NR" => apply_nr_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		_ => Err(Error::BadRequest {
			message: format!("unsupported direct page '{page_id}'"),
		}),
	}
}

async fn apply_direct_page_changes_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	changes: &BTreeMap<String, CaseEditorFieldPatch>,
) -> Result<()> {
	let rows = match page_id {
		"RP" => row_array_payload_from_changes(
			page_id,
			"primarySources",
			changes,
			&[
				("reporterTitle", "reporterTitle"),
				("reporterTitleNullFlavor", "reporterTitleNullFlavor"),
				("reporterGivenName", "reporterGivenName"),
				("reporterGivenNameNullFlavor", "reporterGivenNameNullFlavor"),
				("reporterMiddleName", "reporterMiddleName"),
				(
					"reporterMiddleNameNullFlavor",
					"reporterMiddleNameNullFlavor",
				),
				("reporterFamilyName", "reporterFamilyName"),
				(
					"reporterFamilyNameNullFlavor",
					"reporterFamilyNameNullFlavor",
				),
				("reporterOrganization", "reporterOrganization"),
				(
					"reporterOrganizationNullFlavor",
					"reporterOrganizationNullFlavor",
				),
				("reporterDepartment", "reporterDepartment"),
				(
					"reporterDepartmentNullFlavor",
					"reporterDepartmentNullFlavor",
				),
				("reporterStreet", "reporterStreet"),
				("reporterStreetNullFlavor", "reporterStreetNullFlavor"),
				("reporterCity", "reporterCity"),
				("reporterCityNullFlavor", "reporterCityNullFlavor"),
				("reporterState", "reporterState"),
				("reporterStateNullFlavor", "reporterStateNullFlavor"),
				("reporterPostcode", "reporterPostcode"),
				("reporterPostcodeNullFlavor", "reporterPostcodeNullFlavor"),
				("reporterTelephone", "reporterTelephone"),
				("reporterTelephoneNullFlavor", "reporterTelephoneNullFlavor"),
				("reporterCountry", "reporterCountry"),
				("reporterCountryNullFlavor", "reporterCountryNullFlavor"),
				("reporterEmail", "reporterEmail"),
				("reporterEmailNullFlavor", "reporterEmailNullFlavor"),
				("qualification", "qualification"),
				("qualificationNullFlavor", "qualificationNullFlavor"),
				("qualificationKr1", "qualificationKr1"),
				(
					"primarySourceForRegulatoryPurposes",
					"primarySourceForRegulatoryPurposes",
				),
			],
		)?,
		"SD" => direct_sd_rows_from_changes(page_id, changes)?,
		"LR" => row_array_payload_from_changes(
			page_id,
			"literatureReferences",
			changes,
			&[
				("literatureReference", "referenceText"),
				("referenceText", "referenceText"),
			],
		)?,
		"SI" => row_payload_from_changes(
			page_id,
			"studyInformation",
			changes,
			&[
				("studyName", "studyName"),
				("sponsorStudyNumber", "sponsorStudyNumber"),
				("studyTypeReaction", "studyTypeReaction"),
				("studyTypeReactionKr1", "studyTypeReactionKr1"),
			],
		)?,
		"DM" => row_payload_from_changes(
			page_id,
			"patientInformation",
			changes,
			&[
				("patientInitials", "patientInitials"),
				("patientGivenName", "patientGivenName"),
				("patientFamilyName", "patientFamilyName"),
				("patientSex", "sex"),
				("sex", "sex"),
			],
		)?,
		"NR" => row_payload_from_changes(
			page_id,
			"narrative",
			changes,
			&[
				("caseNarrative", "caseNarrative"),
				("reporterComments", "reporterComments"),
				("senderComments", "senderComments"),
			],
		)?,
		_ => {
			return Err(Error::BadRequest {
				message: format!("unsupported direct page '{page_id}'"),
			})
		}
	};
	apply_direct_page_rows_patch(ctx, mm, case_id, page_id, &rows).await
}

fn direct_sd_rows_from_changes(
	page_id: &str,
	changes: &BTreeMap<String, CaseEditorFieldPatch>,
) -> Result<BTreeMap<String, Value>> {
	let mut rows = BTreeMap::new();
	for (field, patch) in changes {
		let (row_key, target) = match field.as_str() {
			"senderType" => ("senderInformation", "senderType"),
			"senderHealthProfessionalTypeKr1" => {
				("senderInformation", "healthProfessionalTypeKr1")
			}
			"senderOrganization" => ("senderInformation", "organizationName"),
			"senderDepartment" => ("senderInformation", "department"),
			"senderPersonTitle" => ("senderInformation", "personTitle"),
			"senderPersonGivenName" => ("senderInformation", "personGivenName"),
			"senderPersonMiddleName" => ("senderInformation", "personMiddleName"),
			"senderPersonFamilyName" => ("senderInformation", "personFamilyName"),
			"senderStreetAddress" => ("senderInformation", "streetAddress"),
			"senderCity" => ("senderInformation", "city"),
			"senderState" => ("senderInformation", "state"),
			"senderPostcode" => ("senderInformation", "postcode"),
			"senderCountryCode" => ("senderInformation", "countryCode"),
			"senderTelephone" => ("senderInformation", "telephone"),
			"senderFax" => ("senderInformation", "fax"),
			"senderEmail" => ("senderInformation", "email"),
			"receiverOrganization" => ("receiverInformation", "organizationName"),
			"receiverType" => ("receiverInformation", "receiverType"),
			"receiverDepartment" => ("receiverInformation", "department"),
			"receiverStreet" => ("receiverInformation", "streetAddress"),
			"receiverCity" => ("receiverInformation", "city"),
			"receiverState" => ("receiverInformation", "stateProvince"),
			"receiverPostcode" => ("receiverInformation", "postcode"),
			"receiverCountry" => ("receiverInformation", "countryCode"),
			"receiverTelephone" => ("receiverInformation", "telephone"),
			"receiverFax" => ("receiverInformation", "fax"),
			"receiverEmail" => ("receiverInformation", "email"),
			_ => {
				return Err(Error::BadRequest {
					message: format!("unknown {page_id} field '{field}'"),
				})
			}
		};
		let entry = rows
			.entry(row_key.to_string())
			.or_insert_with(|| Value::Object(serde_json::Map::new()));
		let Some(map) = entry.as_object_mut() else {
			return Err(Error::BadRequest {
				message: format!("{page_id}.{row_key} must be an object"),
			});
		};
		map.insert(target.to_string(), patch_json_value(patch));
	}
	Ok(rows)
}

fn datetime_field(
	page_id: &str,
	row: &Map<String, Value>,
	keys: &[&str],
) -> Result<Option<time::OffsetDateTime>> {
	let Some(value) = keys.iter().find_map(|key| row.get(*key)) else {
		return Ok(None);
	};
	if value.is_null() {
		return Ok(None);
	}
	let Some(value) = value.as_str().map(str::trim) else {
		return Err(Error::BadRequest {
			message: format!(
				"{page_id}.{} must be a datetime string or null",
				keys[0]
			),
		});
	};
	if value.is_empty() {
		return Ok(None);
	}
	if let Ok(value) = time::OffsetDateTime::parse(
		value,
		&time::format_description::well_known::Rfc3339,
	) {
		return Ok(Some(value));
	}
	let format =
		time::format_description::parse("[year][month][day][hour][minute][second]")
			.map_err(|err| Error::BadRequest {
				message: format!("failed to initialize E2B datetime parser: {err}"),
			})?;
	time::PrimitiveDateTime::parse(value, &format)
		.map(|value| Some(value.assume_utc()))
		.map_err(|_| Error::BadRequest {
			message: format!(
				"{page_id}.{} must be RFC3339 or YYYYMMDDhhmmss",
				keys[0]
			),
		})
}

async fn apply_rp_page_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	reject_unknown_row_keys(page_id, rows, &["primarySources"])?;
	let Some(value) = rows.get("primarySources") else {
		return Ok(());
	};
	let Some(sources) = value.as_array() else {
		return Err(Error::BadRequest {
			message: format!("{page_id}.primarySources must be an array"),
		});
	};
	for value in sources {
		let source = as_object(page_id, "primarySources", value)?;
		apply_rp_source_patch(ctx, mm, case_id, source).await?;
	}
	Ok(())
}

async fn apply_rp_source_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	source: &serde_json::Map<String, Value>,
) -> Result<()> {
	let update = PrimarySourceForUpdate {
		source_reporter_presave_id: uuid_field(
			source,
			&["sourceReporterPresaveId", "source_reporter_presave_id"],
		),
		reporter_title: string_field(source, &["reporterTitle", "reporter_title"]),
		reporter_title_null_flavor: string_field(
			source,
			&["reporterTitleNullFlavor", "reporter_title_null_flavor"],
		),
		reporter_given_name: string_field(
			source,
			&["reporterGivenName", "reporter_given_name"],
		),
		reporter_given_name_null_flavor: string_field(
			source,
			&[
				"reporterGivenNameNullFlavor",
				"reporter_given_name_null_flavor",
			],
		),
		reporter_middle_name: string_field(
			source,
			&["reporterMiddleName", "reporter_middle_name"],
		),
		reporter_middle_name_null_flavor: string_field(
			source,
			&[
				"reporterMiddleNameNullFlavor",
				"reporter_middle_name_null_flavor",
			],
		),
		reporter_family_name: string_field(
			source,
			&["reporterFamilyName", "reporter_family_name"],
		),
		reporter_family_name_null_flavor: string_field(
			source,
			&[
				"reporterFamilyNameNullFlavor",
				"reporter_family_name_null_flavor",
			],
		),
		organization: string_field(
			source,
			&["reporterOrganization", "organization"],
		),
		organization_null_flavor: string_field(
			source,
			&["reporterOrganizationNullFlavor", "organization_null_flavor"],
		),
		department: string_field(source, &["reporterDepartment", "department"]),
		department_null_flavor: string_field(
			source,
			&["reporterDepartmentNullFlavor", "department_null_flavor"],
		),
		street: string_field(source, &["reporterStreet", "street"]),
		street_null_flavor: string_field(
			source,
			&["reporterStreetNullFlavor", "street_null_flavor"],
		),
		city: string_field(source, &["reporterCity", "city"]),
		city_null_flavor: string_field(
			source,
			&["reporterCityNullFlavor", "city_null_flavor"],
		),
		state: string_field(source, &["reporterState", "state"]),
		state_null_flavor: string_field(
			source,
			&["reporterStateNullFlavor", "state_null_flavor"],
		),
		postcode: string_field(source, &["reporterPostcode", "postcode"]),
		postcode_null_flavor: string_field(
			source,
			&["reporterPostcodeNullFlavor", "postcode_null_flavor"],
		),
		telephone: string_field(source, &["reporterTelephone", "telephone"]),
		telephone_null_flavor: string_field(
			source,
			&["reporterTelephoneNullFlavor", "telephone_null_flavor"],
		),
		country_code: string_field(source, &["reporterCountry", "country_code"]),
		country_code_null_flavor: string_field(
			source,
			&["reporterCountryNullFlavor", "country_code_null_flavor"],
		),
		email: string_field(source, &["reporterEmail", "email"]),
		email_null_flavor: string_field(
			source,
			&["reporterEmailNullFlavor", "email_null_flavor"],
		),
		qualification: string_field(source, &["qualification"]),
		qualification_null_flavor: string_field(
			source,
			&["qualificationNullFlavor", "qualification_null_flavor"],
		),
		qualification_kr1: string_field(
			source,
			&["qualificationKr1", "qualification_kr1"],
		),
		primary_source_regulatory: string_field(
			source,
			&[
				"primarySourceForRegulatoryPurposes",
				"primary_source_regulatory",
			],
		),
	};
	if let Some(id) = uuid_field(source, &["id"]) {
		PrimarySourceBmc::update(ctx, mm, id, update).await?;
	} else {
		PrimarySourceBmc::create(
			ctx,
			mm,
			PrimarySourceForCreate {
				case_id,
				source_reporter_presave_id: update.source_reporter_presave_id,
				sequence_number: i32_field(
					source,
					&["sequenceNumber", "sequence_number"],
				)
				.unwrap_or(1),
				reporter_title: update.reporter_title,
				reporter_title_null_flavor: update.reporter_title_null_flavor,
				reporter_given_name: update.reporter_given_name,
				reporter_given_name_null_flavor: update
					.reporter_given_name_null_flavor,
				reporter_middle_name: update.reporter_middle_name,
				reporter_middle_name_null_flavor: update
					.reporter_middle_name_null_flavor,
				reporter_family_name: update.reporter_family_name,
				reporter_family_name_null_flavor: update
					.reporter_family_name_null_flavor,
				organization: update.organization,
				organization_null_flavor: update.organization_null_flavor,
				department: update.department,
				department_null_flavor: update.department_null_flavor,
				street: update.street,
				street_null_flavor: update.street_null_flavor,
				city: update.city,
				city_null_flavor: update.city_null_flavor,
				state: update.state,
				state_null_flavor: update.state_null_flavor,
				postcode: update.postcode,
				postcode_null_flavor: update.postcode_null_flavor,
				telephone: update.telephone,
				telephone_null_flavor: update.telephone_null_flavor,
				country_code: update.country_code,
				country_code_null_flavor: update.country_code_null_flavor,
				email: update.email,
				email_null_flavor: update.email_null_flavor,
				qualification: update.qualification,
				qualification_null_flavor: update.qualification_null_flavor,
				qualification_kr1: update.qualification_kr1,
				primary_source_regulatory: update.primary_source_regulatory,
			},
		)
		.await?;
	}
	Ok(())
}

async fn apply_sd_page_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	reject_unknown_row_keys(
		page_id,
		rows,
		&[
			"safetyReportIdentification",
			"senderInformation",
			"receiverInformation",
		],
	)?;
	if let Some(sender) = optional_row_object(page_id, rows, "senderInformation")? {
		let update = SenderInformationForUpdate {
			source_sender_presave_id: uuid_field(
				sender,
				&["sourceSenderPresaveId", "source_sender_presave_id"],
			),
			sender_type: string_field(sender, &["senderType", "sender_type"]),
			health_professional_type_kr1: string_field(
				sender,
				&["healthProfessionalTypeKr1", "health_professional_type_kr1"],
			),
			organization_name: string_field(
				sender,
				&["organizationName", "organization_name"],
			),
			department: string_field(sender, &["department"]),
			street_address: string_field(
				sender,
				&["streetAddress", "street_address"],
			),
			city: string_field(sender, &["city"]),
			state: string_field(sender, &["state"]),
			postcode: string_field(sender, &["postcode"]),
			country_code: string_field(sender, &["countryCode", "country_code"]),
			person_title: string_field(sender, &["personTitle", "person_title"]),
			person_given_name: string_field(
				sender,
				&["personGivenName", "person_given_name"],
			),
			person_middle_name: string_field(
				sender,
				&["personMiddleName", "person_middle_name"],
			),
			person_family_name: string_field(
				sender,
				&["personFamilyName", "person_family_name"],
			),
			telephone: string_field(sender, &["telephone"]),
			fax: string_field(sender, &["fax"]),
			email: string_field(sender, &["email"]),
		};
		let existing_sender_id = SenderInformationBmc::list(
			ctx,
			mm,
			Some(vec![SenderInformationFilter {
				case_id: Some(uuid_eq(case_id)),
			}]),
			Some(ListOptions::default()),
		)
		.await?
		.first()
		.map(|row| row.id);
		if let Some(id) = uuid_field(sender, &["id"]).or(existing_sender_id) {
			SenderInformationBmc::update(ctx, mm, id, update).await?;
		} else {
			SenderInformationBmc::create(
				ctx,
				mm,
				SenderInformationForCreate {
					case_id,
					source_sender_presave_id: update.source_sender_presave_id,
					sender_type: update.sender_type,
					health_professional_type_kr1: update
						.health_professional_type_kr1,
					organization_name: update.organization_name,
					department: update.department,
					street_address: update.street_address,
					city: update.city,
					state: update.state,
					postcode: update.postcode,
					country_code: update.country_code,
					person_title: update.person_title,
					person_given_name: update.person_given_name,
					person_middle_name: update.person_middle_name,
					person_family_name: update.person_family_name,
					telephone: update.telephone,
					fax: update.fax,
					email: update.email,
				},
			)
			.await?;
		}
	}
	if let Some(receiver) =
		optional_row_object(page_id, rows, "receiverInformation")?
	{
		let update = ReceiverInformationForUpdate {
			receiver_type: string_field(
				receiver,
				&["receiverType", "receiver_type"],
			),
			organization_name: string_field(
				receiver,
				&["organizationName", "organization_name"],
			),
			department: string_field(receiver, &["department"]),
			street_address: string_field(
				receiver,
				&["streetAddress", "street_address"],
			),
			city: string_field(receiver, &["city"]),
			state_province: string_field(
				receiver,
				&["stateProvince", "state_province"],
			),
			postcode: string_field(receiver, &["postcode"]),
			country_code: string_field(receiver, &["countryCode", "country_code"]),
			telephone: string_field(receiver, &["telephone"]),
			fax: string_field(receiver, &["fax"]),
			email: string_field(receiver, &["email"]),
		};
		if ReceiverInformationBmc::get_by_case_optional(ctx, mm, case_id)
			.await?
			.is_some()
		{
			ReceiverInformationBmc::update_by_case(ctx, mm, case_id, update).await?;
		} else {
			ReceiverInformationBmc::create(
				ctx,
				mm,
				ReceiverInformationForCreate {
					case_id,
					receiver_type: update.receiver_type,
					organization_name: update.organization_name,
					department: update.department,
					street_address: update.street_address,
					city: update.city,
					state_province: update.state_province,
					postcode: update.postcode,
					country_code: update.country_code,
					telephone: update.telephone,
					fax: update.fax,
					email: update.email,
				},
			)
			.await?;
		}
	}
	Ok(())
}

async fn apply_lr_page_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	reject_unknown_row_keys(page_id, rows, &["literatureReferences"])?;
	let Some(value) = rows.get("literatureReferences") else {
		return Ok(());
	};
	let Some(references) = value.as_array() else {
		return Err(Error::BadRequest {
			message: format!("{page_id}.literatureReferences must be an array"),
		});
	};
	for (index, value) in references.iter().enumerate() {
		let reference = as_object(page_id, "literatureReferences", value)?;
		let id = uuid_field(reference, &["id"]);
		if bool_field(reference, &["deleted", "_delete"]) == Some(true) {
			if let Some(id) = id {
				LiteratureReferenceBmc::delete(ctx, mm, id).await?;
			}
			continue;
		}
		let update = LiteratureReferenceForUpdate {
			reference_text: string_field(
				reference,
				&["referenceText", "reference_text"],
			),
			reference_text_null_flavor: string_field(
				reference,
				&["referenceTextNullFlavor", "reference_text_null_flavor"],
			),
			sequence_number: i32_field(
				reference,
				&["sequenceNumber", "sequence_number"],
			),
			document_base64: string_field(
				reference,
				&["documentBase64", "document_base64"],
			),
			media_type: string_field(reference, &["mediaType", "media_type"]),
			representation: string_field(reference, &["representation"]),
			compression: string_field(reference, &["compression"]),
		};
		if let Some(id) = id {
			LiteratureReferenceBmc::update(ctx, mm, id, update).await?;
		} else if let Some(reference_text) = update.reference_text {
			LiteratureReferenceBmc::create(
				ctx,
				mm,
				LiteratureReferenceForCreate {
					case_id,
					reference_text,
					reference_text_null_flavor: update.reference_text_null_flavor,
					sequence_number: update.sequence_number.unwrap_or_else(|| {
						i32::try_from(index + 1).unwrap_or(i32::MAX)
					}),
					document_base64: update.document_base64,
					media_type: update.media_type,
					representation: update.representation,
					compression: update.compression,
				},
			)
			.await?;
		}
	}
	Ok(())
}

async fn apply_si_page_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	reject_unknown_row_keys(
		page_id,
		rows,
		&["studyInformation", "studyRegistrationNumbers"],
	)?;
	let study = optional_row_object(page_id, rows, "studyInformation")?;
	let study_id = if let Some(study) = study {
		let update = StudyInformationForUpdate {
			source_study_presave_id: uuid_field(
				study,
				&["sourceStudyPresaveId", "source_study_presave_id"],
			),
			study_name: string_field(study, &["studyName", "study_name"]),
			study_name_null_flavor: string_field(
				study,
				&["studyNameNullFlavor", "study_name_null_flavor"],
			),
			sponsor_study_number: string_field(
				study,
				&["sponsorStudyNumber", "sponsor_study_number"],
			),
			sponsor_study_number_null_flavor: string_field(
				study,
				&[
					"sponsorStudyNumberNullFlavor",
					"sponsor_study_number_null_flavor",
				],
			),
			study_type_reaction: string_field(
				study,
				&["studyTypeReaction", "study_type_reaction"],
			),
			study_type_reaction_kr1: string_field(
				study,
				&["studyTypeReactionKr1", "study_type_reaction_kr1"],
			),
			fda_ind_number_occurred: string_field(
				study,
				&["fdaIndNumberOccurred", "fda_ind_number_occurred"],
			),
			fda_pre_anda_number_occurred: string_field(
				study,
				&["fdaPreAndaNumberOccurred", "fda_pre_anda_number_occurred"],
			),
		};
		if let Some(id) = uuid_field(study, &["id"]) {
			StudyInformationBmc::update(ctx, mm, id, update).await?;
			id
		} else {
			StudyInformationBmc::create(
				ctx,
				mm,
				StudyInformationForCreate {
					case_id,
					source_study_presave_id: update.source_study_presave_id,
					study_name: update.study_name,
					study_name_null_flavor: update.study_name_null_flavor,
					sponsor_study_number: update.sponsor_study_number,
					sponsor_study_number_null_flavor: update
						.sponsor_study_number_null_flavor,
					study_type_reaction: update.study_type_reaction,
					study_type_reaction_kr1: update.study_type_reaction_kr1,
					fda_ind_number_occurred: update.fda_ind_number_occurred,
					fda_pre_anda_number_occurred: update
						.fda_pre_anda_number_occurred,
				},
			)
			.await?
		}
	} else {
		let studies = StudyInformationBmc::list(
			ctx,
			mm,
			Some(vec![StudyInformationFilter {
				case_id: Some(uuid_eq(case_id)),
			}]),
			Some(ListOptions::default()),
		)
		.await?;
		let Some(study) = studies.into_iter().min_by_key(|study| study.created_at)
		else {
			if rows.contains_key("studyRegistrationNumbers") {
				return Err(Error::BadRequest {
					message: format!(
						"{page_id}.studyInformation is required before child rows"
					),
				});
			}
			return Ok(());
		};
		study.id
	};

	if let Some(value) = rows.get("studyRegistrationNumbers") {
		let Some(registrations) = value.as_array() else {
			return Err(Error::BadRequest {
				message: format!(
					"{page_id}.studyRegistrationNumbers must be an array"
				),
			});
		};
		for (index, value) in registrations.iter().enumerate() {
			let registration =
				as_object(page_id, "studyRegistrationNumbers", value)?;
			let id = uuid_field(registration, &["id"]);
			if bool_field(registration, &["deleted", "_delete"]) == Some(true) {
				if let Some(id) = id {
					StudyRegistrationNumberBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let update = StudyRegistrationNumberForUpdate {
				registration_number: string_field(
					registration,
					&["registrationNumber", "registration_number"],
				),
				registration_number_null_flavor: string_field(
					registration,
					&[
						"registrationNumberNullFlavor",
						"registration_number_null_flavor",
					],
				),
				country_code: string_field(
					registration,
					&["countryCode", "country_code"],
				),
				country_code_null_flavor: string_field(
					registration,
					&["countryCodeNullFlavor", "country_code_null_flavor"],
				),
				sequence_number: i32_field(
					registration,
					&["sequenceNumber", "sequence_number"],
				),
			};
			if let Some(id) = id {
				StudyRegistrationNumberBmc::update(ctx, mm, id, update).await?;
			} else if let Some(registration_number) = update.registration_number {
				StudyRegistrationNumberBmc::create(
					ctx,
					mm,
					StudyRegistrationNumberForCreate {
						study_information_id: study_id,
						registration_number,
						registration_number_null_flavor: update
							.registration_number_null_flavor,
						country_code: update.country_code,
						country_code_null_flavor: update.country_code_null_flavor,
						sequence_number: update.sequence_number.unwrap_or_else(
							|| i32::try_from(index + 1).unwrap_or(i32::MAX),
						),
					},
				)
				.await?;
			}
		}
	}

	if let Some(study) = study {
		if let Some(value) = study
			.get("fdaCrossReportedIndNumbers")
			.or_else(|| study.get("fda_cross_reported_ind_numbers"))
		{
			let Some(numbers) = value.as_array() else {
				return Err(Error::BadRequest {
					message: format!(
						"{page_id}.studyInformation.fdaCrossReportedIndNumbers must be an array"
					),
				});
			};
			for (index, value) in numbers.iter().enumerate() {
				let number = as_object(
					page_id,
					"studyInformation.fdaCrossReportedIndNumbers",
					value,
				)?;
				let id = uuid_field(number, &["id"]);
				if bool_field(number, &["deleted", "_delete"]) == Some(true) {
					if let Some(id) = id {
						StudyFdaCrossReportedIndBmc::delete(ctx, mm, id).await?;
					}
					continue;
				}
				let update = StudyFdaCrossReportedIndForUpdate {
					ind_number: string_field(number, &["indNumber", "ind_number"]),
					ind_number_null_flavor: string_field(
						number,
						&["indNumberNullFlavor", "ind_number_null_flavor"],
					),
					sequence_number: i32_field(
						number,
						&["sequenceNumber", "sequence_number"],
					),
				};
				if let Some(id) = id {
					StudyFdaCrossReportedIndBmc::update(ctx, mm, id, update).await?;
				} else {
					StudyFdaCrossReportedIndBmc::create(
						ctx,
						mm,
						StudyFdaCrossReportedIndForCreate {
							study_information_id: study_id,
							ind_number: update.ind_number,
							ind_number_null_flavor: update.ind_number_null_flavor,
							sequence_number: update.sequence_number.unwrap_or_else(
								|| i32::try_from(index + 1).unwrap_or(i32::MAX),
							),
						},
					)
					.await?;
				}
			}
		}
	}
	Ok(())
}

async fn apply_dm_page_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	reject_unknown_row_keys(
		page_id,
		rows,
		&[
			"patientInformation",
			"patientIdentifiers",
			"medicalHistoryEpisodes",
			"deathInfo",
			"reportedCauses",
			"autopsyCauses",
			"parentInfo",
			"parentMedicalHistory",
			"parentPastDrugs",
		],
	)?;
	let Some(patient) = optional_row_object(page_id, rows, "patientInformation")?
	else {
		return Ok(());
	};
	fn value_at_path<'a>(
		row: &'a Map<String, Value>,
		paths: &[&str],
	) -> Option<&'a Value> {
		'paths: for path in paths {
			let mut segments = path.split('.');
			let Some(first) = segments.next() else {
				continue;
			};
			let Some(mut value) = row.get(first) else {
				continue;
			};
			for segment in segments {
				let Some(next) =
					value.as_object().and_then(|object| object.get(segment))
				else {
					continue 'paths;
				};
				value = next;
			}
			return Some(value);
		}
		None
	}
	fn decimal_field(
		page_id: &str,
		row: &Map<String, Value>,
		paths: &[&str],
	) -> Result<Option<Decimal>> {
		let Some(value) = value_at_path(row, paths) else {
			return Ok(None);
		};
		if value.is_null() {
			return Ok(None);
		}
		Decimal::from_str(&value.to_string())
			.map(Some)
			.map_err(|_| Error::BadRequest {
				message: format!(
					"{page_id}.{} must be a decimal number or null",
					paths[0]
				),
			})
	}
	fn date_field(
		page_id: &str,
		row: &Map<String, Value>,
		paths: &[&str],
	) -> Result<Option<sqlx::types::time::Date>> {
		let Some(value) = value_at_path(row, paths) else {
			return Ok(None);
		};
		serde_json::from_value::<CiDatePatchValue>(json!({"value": value}))
			.map(|parsed| parsed.value)
			.map_err(|err| Error::BadRequest {
				message: format!(
					"{page_id}.{} must be an E2B date or null: {err}",
					paths[0]
				),
			})
	}
	fn boolean_field(row: &Map<String, Value>, paths: &[&str]) -> Option<bool> {
		value_at_path(row, paths).and_then(Value::as_bool)
	}
	fn nested_string_field(
		row: &Map<String, Value>,
		paths: &[&str],
	) -> Option<String> {
		value_at_path(row, paths)
			.filter(|value| !value.is_null())
			.map(|value| {
				value
					.as_str()
					.map(ToOwned::to_owned)
					.unwrap_or_else(|| value.to_string())
			})
	}
	let update = PatientInformationForUpdate {
		patient_initials: string_field(
			patient,
			&["patientInitials", "patient_initials"],
		),
		patient_initials_null_flavor: string_field(
			patient,
			&["patientInitialsNullFlavor", "patient_initials_null_flavor"],
		),
		birth_date: date_field(
			page_id,
			patient,
			&["patientBirthDate", "birth_date"],
		)?,
		birth_date_null_flavor: string_field(
			patient,
			&["birthDateNullFlavor", "birth_date_null_flavor"],
		),
		age_at_time_of_onset: decimal_field(
			page_id,
			patient,
			&[
				"patientAge.value",
				"ageAtTimeOfOnset",
				"age_at_time_of_onset",
			],
		)?,
		age_at_time_of_onset_null_flavor: string_field(
			patient,
			&[
				"ageAtTimeOfOnsetNullFlavor",
				"age_at_time_of_onset_null_flavor",
			],
		),
		age_unit: nested_string_field(
			patient,
			&["patientAge.unit", "ageUnit", "age_unit"],
		),
		gestation_period: decimal_field(
			page_id,
			patient,
			&["gestationPeriod.value", "gestation_period"],
		)?,
		gestation_period_unit: nested_string_field(
			patient,
			&[
				"gestationPeriod.unit",
				"gestationPeriodUnit",
				"gestation_period_unit",
			],
		),
		age_group: string_field(
			patient,
			&["patientAgeGroup", "ageGroup", "age_group"],
		),
		weight_kg: decimal_field(
			page_id,
			patient,
			&["patientWeight.value", "weightKg", "weight_kg"],
		)?,
		weight_kg_null_flavor: string_field(
			patient,
			&["weightKgNullFlavor", "weight_kg_null_flavor"],
		),
		height_cm: decimal_field(
			page_id,
			patient,
			&["patientHeight.value", "heightCm", "height_cm"],
		)?,
		height_cm_null_flavor: string_field(
			patient,
			&["heightCmNullFlavor", "height_cm_null_flavor"],
		),
		sex: string_field(patient, &["patientSex", "sex"]),
		sex_null_flavor: string_field(
			patient,
			&["sexNullFlavor", "sex_null_flavor"],
		),
		race_code: string_field(patient, &["raceCode", "race_code"]),
		race_code_null_flavor: string_field(
			patient,
			&["raceCodeNullFlavor", "race_code_null_flavor"],
		),
		ethnicity_code: string_field(patient, &["ethnicityCode", "ethnicity_code"]),
		ethnicity_code_null_flavor: string_field(
			patient,
			&["ethnicityCodeNullFlavor", "ethnicity_code_null_flavor"],
		),
		last_menstrual_period_date: date_field(
			page_id,
			patient,
			&["lastMenstrualPeriodDate", "last_menstrual_period_date"],
		)?,
		last_menstrual_period_date_null_flavor: string_field(
			patient,
			&[
				"lastMenstrualPeriodDateNullFlavor",
				"last_menstrual_period_date_null_flavor",
			],
		),
		medical_history_text: string_field(
			patient,
			&["medicalHistoryText", "medical_history_text"],
		),
		medical_history_text_null_flavor: string_field(
			patient,
			&[
				"medicalHistoryTextNullFlavor",
				"medical_history_text_null_flavor",
			],
		),
		concomitant_therapy: boolean_field(
			patient,
			&["concomitantTherapies", "concomitant_therapy"],
		),
	};
	match PatientInformationBmc::get_by_case(ctx, mm, case_id).await {
		Ok(_) => {
			PatientInformationBmc::update_by_case(ctx, mm, case_id, update).await?
		}
		Err(lib_core::model::Error::EntityUuidNotFound { .. }) => {
			PatientInformationBmc::create(
				ctx,
				mm,
				PatientInformationForCreate {
					case_id,
					patient_initials: update.patient_initials,
					patient_initials_null_flavor: update
						.patient_initials_null_flavor,
					birth_date: update.birth_date,
					birth_date_null_flavor: update.birth_date_null_flavor,
					age_at_time_of_onset: update.age_at_time_of_onset,
					age_at_time_of_onset_null_flavor: update
						.age_at_time_of_onset_null_flavor,
					age_unit: update.age_unit,
					gestation_period: update.gestation_period,
					gestation_period_unit: update.gestation_period_unit,
					age_group: update.age_group,
					weight_kg: update.weight_kg,
					weight_kg_null_flavor: update.weight_kg_null_flavor,
					height_cm: update.height_cm,
					height_cm_null_flavor: update.height_cm_null_flavor,
					sex: update.sex,
					sex_null_flavor: update.sex_null_flavor,
					race_code: update.race_code,
					race_code_null_flavor: update.race_code_null_flavor,
					ethnicity_code: update.ethnicity_code,
					ethnicity_code_null_flavor: update.ethnicity_code_null_flavor,
					last_menstrual_period_date: update.last_menstrual_period_date,
					last_menstrual_period_date_null_flavor: update
						.last_menstrual_period_date_null_flavor,
					medical_history_text: update.medical_history_text,
					medical_history_text_null_flavor: update
						.medical_history_text_null_flavor,
					concomitant_therapy: update.concomitant_therapy,
				},
			)
			.await?;
		}
		Err(err) => return Err(err.into()),
	}
	Ok(())
}

async fn apply_nr_page_rows_patch(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	rows: &BTreeMap<String, Value>,
) -> Result<()> {
	reject_unknown_row_keys(
		page_id,
		rows,
		&["narrative", "senderDiagnoses", "caseSummaryInformation"],
	)?;
	let Some(narrative) = optional_row_object(page_id, rows, "narrative")? else {
		return Ok(());
	};
	let case_narrative =
		string_field(narrative, &["caseNarrative", "case_narrative"]);
	let update = NarrativeInformationForUpdate {
		source_narrative_presave_id: uuid_field(
			narrative,
			&["sourceNarrativePresaveId", "source_narrative_presave_id"],
		),
		case_narrative: case_narrative.clone(),
		reporter_comments: string_field(
			narrative,
			&["reporterComments", "reporter_comments"],
		),
		sender_comments: string_field(
			narrative,
			&["senderComments", "sender_comments"],
		),
		additional_information: string_field(
			narrative,
			&["additionalInformation", "additional_information"],
		),
	};
	match NarrativeInformationBmc::get_by_case_optional(ctx, mm, case_id).await? {
		Some(_) => {
			NarrativeInformationBmc::update_by_case(ctx, mm, case_id, update).await?
		}
		None => {
			let Some(case_narrative) = case_narrative else {
				return Ok(());
			};
			NarrativeInformationBmc::create(
				ctx,
				mm,
				NarrativeInformationForCreate {
					case_id,
					source_narrative_presave_id: update.source_narrative_presave_id,
					case_narrative,
					reporter_comments: update.reporter_comments,
					sender_comments: update.sender_comments,
					additional_information: update.additional_information,
				},
			)
			.await?;
		}
	}
	Ok(())
}

async fn load_editor_rp_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Value> {
	let primary_sources = PrimarySourceBmc::list(
		ctx,
		mm,
		Some(vec![PrimarySourceFilter {
			case_id: Some(uuid_eq(case_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?
	.into_iter()
	.map(CaseEditorRpPrimarySourceDto::from)
	.collect::<Vec<_>>();

	Ok(json!({ "primarySources": primary_sources }))
}

pub async fn get_editor_rp(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, PRIMARY_SOURCE_LIST)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	Ok(direct_section_response(
		case_id,
		load_editor_rp_data(&ctx, &mm, case_id).await?,
	))
}

direct_page_projection_handler!(
	get_editor_rp_page_projection,
	"RP",
	load_editor_rp_data,
	[PRIMARY_SOURCE_LIST],
);

async fn load_editor_sd_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Value> {
	let sender_information = SenderInformationBmc::list(
		ctx,
		mm,
		Some(vec![SenderInformationFilter {
			case_id: Some(uuid_eq(case_id)),
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let sender = sender_information.first().cloned();
	let receiver =
		ReceiverInformationBmc::get_by_case_optional(ctx, mm, case_id).await?;

	let safety_report_identification = json!({
		"senderInformationId": sender.as_ref().map(|row| row.id),
		"senderType": sender.as_ref().and_then(|row| row.sender_type.clone()),
		"senderHealthProfessionalTypeKr1": sender
			.as_ref()
			.and_then(|row| row.health_professional_type_kr1.clone()),
		"senderOrganization": sender
			.as_ref()
			.and_then(|row| row.organization_name.clone()),
		"senderDepartment": sender.as_ref().and_then(|row| row.department.clone()),
		"senderPersonTitle": sender.as_ref().and_then(|row| row.person_title.clone()),
		"senderPersonGivenName": sender
			.as_ref()
			.and_then(|row| row.person_given_name.clone()),
		"senderPersonMiddleName": sender
			.as_ref()
			.and_then(|row| row.person_middle_name.clone()),
		"senderPersonFamilyName": sender
			.as_ref()
			.and_then(|row| row.person_family_name.clone()),
		"senderStreetAddress": sender
			.as_ref()
			.and_then(|row| row.street_address.clone()),
		"senderCity": sender.as_ref().and_then(|row| row.city.clone()),
		"senderState": sender.as_ref().and_then(|row| row.state.clone()),
		"senderPostcode": sender.as_ref().and_then(|row| row.postcode.clone()),
		"senderCountryCode": sender.as_ref().and_then(|row| row.country_code.clone()),
		"senderTelephone": sender.as_ref().and_then(|row| row.telephone.clone()),
		"senderFax": sender.as_ref().and_then(|row| row.fax.clone()),
		"senderEmail": sender.as_ref().and_then(|row| row.email.clone()),
		"receiverType": receiver.as_ref().and_then(|row| row.receiver_type.clone()),
		"receiverOrganization": receiver
			.as_ref()
			.and_then(|row| row.organization_name.clone()),
		"receiverDepartment": receiver.as_ref().and_then(|row| row.department.clone()),
		"receiverStreet": receiver.as_ref().and_then(|row| row.street_address.clone()),
		"receiverCity": receiver.as_ref().and_then(|row| row.city.clone()),
		"receiverState": receiver.as_ref().and_then(|row| row.state_province.clone()),
		"receiverPostcode": receiver.as_ref().and_then(|row| row.postcode.clone()),
		"receiverCountry": receiver.as_ref().and_then(|row| row.country_code.clone()),
		"receiverTelephone": receiver.as_ref().and_then(|row| row.telephone.clone()),
		"receiverFax": receiver.as_ref().and_then(|row| row.fax.clone()),
		"receiverEmail": receiver.as_ref().and_then(|row| row.email.clone()),
	});
	Ok(json!({
		"safetyReportIdentification": safety_report_identification,
		"senderInformation": sender_information,
		"sender": sender,
		"receiverInformation": receiver,
	}))
}

pub async fn get_editor_sd(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, SAFETY_REPORT_READ)?;
	require_permission(&ctx, SENDER_INFORMATION_LIST)?;
	require_permission(&ctx, RECEIVER_READ)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	Ok(direct_section_response(
		case_id,
		load_editor_sd_data(&ctx, &mm, case_id).await?,
	))
}

direct_page_projection_handler!(
	get_editor_sd_page_projection,
	"SD",
	load_editor_sd_data,
	[SAFETY_REPORT_READ, SENDER_INFORMATION_LIST, RECEIVER_READ],
);

async fn load_editor_lr_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Value> {
	let literature_references = LiteratureReferenceBmc::list(
		ctx,
		mm,
		Some(vec![LiteratureReferenceFilter {
			case_id: Some(uuid_eq(case_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?;

	Ok(json!({
		"literatureReferences": literature_references
			.into_iter()
			.map(|row| json!({
				"id": row.id,
				"sequenceNumber": row.sequence_number,
				"referenceText": row.reference_text,
				"referenceTextNullFlavor": row.reference_text_null_flavor,
				"documentBase64": row.document_base64,
				"mediaType": row.media_type,
				"representation": row.representation,
				"compression": row.compression,
				"deleted": row.deleted,
			}))
			.collect::<Vec<_>>()
	}))
}

pub async fn get_editor_lr(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, LITERATURE_REFERENCE_LIST)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	Ok(direct_section_response(
		case_id,
		load_editor_lr_data(&ctx, &mm, case_id).await?,
	))
}

direct_page_projection_handler!(
	get_editor_lr_page_projection,
	"LR",
	load_editor_lr_data,
	[LITERATURE_REFERENCE_LIST],
);

async fn load_editor_si_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Value> {
	let mut studies = StudyInformationBmc::list(
		ctx,
		mm,
		Some(vec![StudyInformationFilter {
			case_id: Some(uuid_eq(case_id)),
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	studies.sort_by_key(|study| study.created_at);
	let study_information = studies.into_iter().next();
	let (study_registration_numbers, fda_cross_reported_ind_numbers) =
		if let Some(ref study) = study_information {
			let registrations = StudyRegistrationNumberBmc::list(
				ctx,
				mm,
				Some(vec![StudyRegistrationNumberFilter {
					study_information_id: Some(uuid_eq(study.id)),
					..Default::default()
				}]),
				Some(ListOptions::default()),
			)
			.await?;
			let cross_reported = StudyFdaCrossReportedIndBmc::list(
				ctx,
				mm,
				Some(vec![StudyFdaCrossReportedIndFilter {
					study_information_id: Some(uuid_eq(study.id)),
					..Default::default()
				}]),
				Some(ListOptions::default()),
			)
			.await?;
			(registrations, cross_reported)
		} else {
			(Vec::new(), Vec::new())
		};
	let study_information = study_information.map(|study| {
		let mut value = json!(study);
		value
			.as_object_mut()
			.expect("serialized study information is an object")
			.insert(
				"fdaCrossReportedIndNumbers".to_string(),
				json!(fda_cross_reported_ind_numbers),
			);
		value
	});

	Ok(json!({
		"studyInformation": study_information,
		"studyRegistrationNumbers": study_registration_numbers,
	}))
}

pub async fn get_editor_si(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, STUDY_INFORMATION_LIST)?;
	require_permission(&ctx, STUDY_REGISTRATION_LIST)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	Ok(direct_section_response(
		case_id,
		load_editor_si_data(&ctx, &mm, case_id).await?,
	))
}

direct_page_projection_handler!(
	get_editor_si_page_projection,
	"SI",
	load_editor_si_data,
	[STUDY_INFORMATION_LIST, STUDY_REGISTRATION_LIST],
);

async fn load_editor_dm_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Value> {
	let Some(patient) =
		(match PatientInformationBmc::get_by_case(ctx, mm, case_id).await {
			Ok(entity) => Some(entity),
			Err(lib_core::model::Error::EntityUuidNotFound { .. }) => None,
			Err(err) => return Err(err.into()),
		})
	else {
		return Ok(json!({
			"patientInformation": null,
			"patientIdentifiers": [],
			"medicalHistoryEpisodes": [],
			"deathInfo": null,
			"reportedCauses": [],
			"autopsyCauses": [],
			"parentInfo": null,
			"parentMedicalHistory": [],
			"parentPastDrugs": [],
		}));
	};

	let patient_id = patient.id;
	let patient_identifiers = PatientIdentifierBmc::list(
		ctx,
		mm,
		Some(vec![PatientIdentifierFilter {
			patient_id: Some(uuid_eq(patient_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let medical_history_episodes = MedicalHistoryEpisodeBmc::list(
		ctx,
		mm,
		Some(vec![MedicalHistoryEpisodeFilter {
			patient_id: Some(uuid_eq(patient_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let parent_information_rows = ParentInformationBmc::list(
		ctx,
		mm,
		Some(vec![ParentInformationFilter {
			patient_id: Some(uuid_eq(patient_id)),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let mut parents = Vec::new();
	let mut parent_medical_history = Vec::new();
	let mut parent_past_drugs = Vec::new();
	for parent in &parent_information_rows {
		let medical_history = ParentMedicalHistoryBmc::list(
			ctx,
			mm,
			Some(vec![ParentMedicalHistoryFilter {
				parent_id: Some(uuid_eq(parent.id)),
				..Default::default()
			}]),
			Some(ListOptions::default()),
		)
		.await?;
		let past_drug_history = ParentPastDrugHistoryBmc::list(
			ctx,
			mm,
			Some(vec![ParentPastDrugHistoryFilter {
				parent_id: Some(uuid_eq(parent.id)),
				..Default::default()
			}]),
			Some(ListOptions::default()),
		)
		.await?;
		let mut parent_with_children = json!(parent);
		if let Value::Object(ref mut map) = parent_with_children {
			map.insert("medicalHistory".to_string(), json!(medical_history));
			map.insert("pastDrugHistory".to_string(), json!(past_drug_history));
			map.insert("pastDrugs".to_string(), json!(past_drug_history));
		}
		parent_medical_history.extend(medical_history);
		parent_past_drugs.extend(past_drug_history);
		parents.push(parent_with_children);
	}
	let death_information = PatientDeathInformationBmc::list(
		ctx,
		mm,
		Some(vec![PatientDeathInformationFilter {
			patient_id: Some(uuid_eq(patient_id)),
		}]),
		Some(ListOptions::default()),
	)
	.await?;
	let mut reported_causes = Vec::new();
	let mut autopsy_causes = Vec::new();
	for death_info in &death_information {
		reported_causes.extend(
			ReportedCauseOfDeathBmc::list(
				ctx,
				mm,
				Some(vec![ReportedCauseOfDeathFilter {
					death_info_id: Some(uuid_eq(death_info.id)),
					..Default::default()
				}]),
				Some(ListOptions::default()),
			)
			.await?,
		);
		autopsy_causes.extend(
			AutopsyCauseOfDeathBmc::list(
				ctx,
				mm,
				Some(vec![AutopsyCauseOfDeathFilter {
					death_info_id: Some(uuid_eq(death_info.id)),
					..Default::default()
				}]),
				Some(ListOptions::default()),
			)
			.await?,
		);
	}
	let death_info = death_information.into_iter().next();
	let parent_info = parent_information_rows.into_iter().next();
	let mut patient_projection = json!(patient);
	if let Value::Object(ref mut map) = patient_projection {
		map.insert("birth_date".to_string(), json!(ci_date(patient.birth_date)));
		map.insert(
			"last_menstrual_period_date".to_string(),
			json!(ci_date(patient.last_menstrual_period_date)),
		);
	}

	Ok(json!({
		"patientInformation": patient_projection,
		"patientIdentifiers": patient_identifiers,
		"medicalHistoryEpisodes": medical_history_episodes,
		"deathInfo": death_info,
		"reportedCauses": reported_causes,
		"autopsyCauses": autopsy_causes,
		"parentInfo": parent_info,
		"parentMedicalHistory": parent_medical_history,
		"parentPastDrugs": parent_past_drugs,
		"parents": parents,
	}))
}

pub async fn get_editor_dm(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, PATIENT_READ)?;
	require_permission(&ctx, PATIENT_IDENTIFIER_LIST)?;
	require_permission(&ctx, MEDICAL_HISTORY_LIST)?;
	require_permission(&ctx, PATIENT_DEATH_LIST)?;
	require_permission(&ctx, DEATH_CAUSE_LIST)?;
	require_permission(&ctx, PARENT_INFORMATION_LIST)?;
	require_permission(&ctx, PARENT_MEDICAL_HISTORY_LIST)?;
	require_permission(&ctx, PARENT_PAST_DRUG_LIST)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	Ok(direct_section_response(
		case_id,
		load_editor_dm_data(&ctx, &mm, case_id).await?,
	))
}

direct_page_projection_handler!(
	get_editor_dm_page_projection,
	"DM",
	load_editor_dm_data,
	[
		PATIENT_READ,
		PATIENT_IDENTIFIER_LIST,
		MEDICAL_HISTORY_LIST,
		PATIENT_DEATH_LIST,
		DEATH_CAUSE_LIST,
		PARENT_INFORMATION_LIST,
		PARENT_MEDICAL_HISTORY_LIST,
		PARENT_PAST_DRUG_LIST
	],
);

async fn load_editor_nr_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Value> {
	let narrative =
		NarrativeInformationBmc::get_by_case_optional(ctx, mm, case_id).await?;
	let (sender_diagnoses, case_summary_information) =
		if let Some(ref narrative) = narrative {
			let sender_diagnoses = SenderDiagnosisBmc::list(
				ctx,
				mm,
				Some(vec![SenderDiagnosisFilter {
					narrative_id: Some(uuid_eq(narrative.id)),
					..Default::default()
				}]),
				Some(ListOptions::default()),
			)
			.await?;
			let case_summary_information = CaseSummaryInformationBmc::list(
				ctx,
				mm,
				Some(vec![CaseSummaryInformationFilter {
					narrative_id: Some(uuid_eq(narrative.id)),
					..Default::default()
				}]),
				Some(ListOptions::default()),
			)
			.await?;
			(sender_diagnoses, case_summary_information)
		} else {
			(Vec::new(), Vec::new())
		};

	Ok(json!({
		"narrative": narrative,
		"senderDiagnoses": sender_diagnoses,
		"caseSummaryInformation": case_summary_information,
	}))
}

pub async fn get_editor_nr(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	require_permission(&ctx, CASE_READ)?;
	require_permission(&ctx, NARRATIVE_READ)?;
	require_permission(&ctx, SENDER_DIAGNOSIS_LIST)?;
	require_permission(&ctx, CASE_SUMMARY_LIST)?;
	lib_rest_core::require_case_read_allowed(&ctx, &mm, case_id).await?;

	Ok(direct_section_response(
		case_id,
		load_editor_nr_data(&ctx, &mm, case_id).await?,
	))
}

direct_page_projection_handler!(
	get_editor_nr_page_projection,
	"NR",
	load_editor_nr_data,
	[NARRATIVE_READ, SENDER_DIAGNOSIS_LIST, CASE_SUMMARY_LIST],
);

#[cfg(test)]
mod tests {
	use super::*;

	fn changes(field: &str, value: Value) -> BTreeMap<String, CaseEditorFieldPatch> {
		let patch = serde_json::from_value(json!({ "value": value }))
			.expect("field patch should deserialize");
		BTreeMap::from([(field.to_string(), patch)])
	}

	#[test]
	fn ci_gate_rejects_invalid_inline_value() {
		let error = validate_direct_changes(
			"CI",
			&changes("reportType", Value::String("9".to_string())),
		)
		.expect_err("invalid report type should fail");
		assert!(format!("{error:?}").contains("ICH.C.1.3.ALLOWED.VALUE"));
	}

	#[test]
	fn ci_gate_validates_null_flavor_values() {
		assert!(validate_direct_changes(
			"CI",
			&changes(
				"fulfilExpeditedCriteriaNullFlavor",
				Value::String("NI".to_string()),
			)
		)
		.is_ok());
		let error = validate_direct_changes(
			"CI",
			&changes(
				"fulfilExpeditedCriteriaNullFlavor",
				Value::String("BAD".to_string()),
			),
		)
		.expect_err("invalid null flavor should fail");
		assert!(format!("{error:?}").contains("ICH.C.1.7.NULLFLAVOR.ALLOWED"));
	}

	#[test]
	fn ci_gate_rejects_non_primitive_patch_values() {
		let error = validate_direct_changes(
			"CI",
			&changes("reportType", json!({ "nested": true })),
		)
		.expect_err("object report type should fail");
		assert!(format!("{error:?}").contains("ICH.C.1.3.LENGTH.MAX"));
	}
}
