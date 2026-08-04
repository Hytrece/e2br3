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
			email: source.email,
			email_null_flavor: source.email_null_flavor,
			qualification: source.qualification,
			qualification_null_flavor: source.qualification_null_flavor,
			qualification_kr1: source.qualification_kr1,
			primary_source_regulatory: source.primary_source_regulatory,
		}
	}
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
				file_name: row.file_name,
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
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/CI",
		move |ctx, mm| {
			Box::pin(async move {
				Ok(direct_section_response(
					case_id,
					load_editor_ci_data(ctx, mm, case_id).await?,
				))
			})
		},
	)
	.await
}

/// GET /api/cases/{case_id}/editor/pages/CI
pub async fn get_editor_ci_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Query(query): Query<CaseEditorPageProjectionQuery>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/CI",
		move |ctx, mm| {
			Box::pin(async move {
				let projection = direct_page_projection_response(
					ctx,
					mm,
					case_id,
					"CI",
					query_authorities_csv(&query)?,
					load_editor_ci_data(ctx, mm, case_id).await?,
				)
				.await?;
				Ok((axum::http::StatusCode::OK, Json(projection)))
			})
		},
	)
	.await
}

#[derive(Deserialize)]
struct CiDatePatchValue {
	#[serde(
		default,
		deserialize_with = "lib_core::serde::flex_date::deserialize_option_date"
	)]
	value: Option<sqlx::types::time::Date>,
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
	file_name: Option<String>,
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
	if rows.contains_key("messageHeader") {
		return Err(Error::BadRequest {
			message: "CI.messageHeader row patches are not supported".to_string(),
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

	let contract_row = rows
		.iter()
		.map(|(key, value)| (key.clone(), value.clone()))
		.collect::<Map<String, Value>>();
	validate_row_payload("CI", "CI", &contract_row, None)?;

	if let Some(row) = optional_row_object("CI", rows, "safetyReportIdentification")?
	{
		fn patch_string(
			row: &Map<String, Value>,
			key: &str,
		) -> Result<PatchValue<String>> {
			match row.get(key) {
				None => Ok(PatchValue::Missing),
				Some(Value::Null) => Ok(PatchValue::Null),
				Some(Value::String(value)) => Ok(PatchValue::Value(value.clone())),
				Some(_) => Err(Error::BadRequest {
					message: format!(
						"CI.safetyReportIdentification.{key} must be a string or null"
					),
				}),
			}
		}
		fn patch_bool(
			row: &Map<String, Value>,
			key: &str,
		) -> Result<PatchValue<bool>> {
			match row.get(key) {
				None => Ok(PatchValue::Missing),
				Some(Value::Null) => Ok(PatchValue::Null),
				Some(Value::Bool(value)) => Ok(PatchValue::Value(*value)),
				Some(_) => Err(Error::BadRequest {
					message: format!(
						"CI.safetyReportIdentification.{key} must be a boolean or null"
					),
				}),
			}
		}
		fn date(
			row: &Map<String, Value>,
			key: &str,
		) -> Result<Option<sqlx::types::time::Date>> {
			let value = row.get(key).cloned().unwrap_or(Value::Null);
			serde_json::from_value::<CiDatePatchValue>(json!({ "value": value }))
				.map(|value| value.value)
				.map_err(|err| Error::BadRequest {
					message: format!(
						"CI.safetyReportIdentification.{key} must be an E2B date or null: {err}"
					),
				})
		}

		SafetyReportIdentificationBmc::update_by_case(
			ctx,
			mm,
			case_id,
			SafetyReportIdentificationForUpdate {
				safety_report_id: string_field(row, &["safetyReportId"]),
				version: i32_field(row, &["version"]),
				transmission_date: string_field(row, &["transmissionDate"]),
				report_type: patch_string(row, "reportType")?,
				date_first_received_from_source: date(
					row,
					"dateFirstReceivedFromSource",
				)?,
				date_of_most_recent_information: date(
					row,
					"dateOfMostRecentInformation",
				)?,
				fulfil_expedited_criteria: patch_bool(
					row,
					"fulfilExpeditedCriteria",
				)?,
				fulfil_expedited_criteria_null_flavor: None,
				local_criteria_report_type: patch_string(
					row,
					"localCriteriaReportType",
				)?,
				combination_product_report_indicator: patch_string(
					row,
					"combinationProductReportIndicator",
				)?,
				combination_product_report_indicator_null_flavor: string_field(
					row,
					&["combinationProductReportIndicatorNullFlavor"],
				),
				worldwide_unique_id: string_field(row, &["worldwideUniqueId"]),
				first_sender_type: string_field(row, &["firstSenderType"]),
				additional_documents_available: bool_field(
					row,
					&["additionalDocumentsAvailable"],
				),
				other_case_identifiers_exist: bool_field(
					row,
					&["otherCaseIdentifiersExist"],
				),
				other_case_identifiers_exist_null_flavor: string_field(
					row,
					&["otherCaseIdentifiersExistNullFlavor"],
				),
				nullification_code: string_field(
					row,
					&["nullificationAmendmentCode"],
				),
				nullification_reason: string_field(row, &["nullificationReason"]),
				receiver_organization: string_field(row, &["receiverOrganization"]),
			},
		)
		.await?;
	}

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
						file_name: patch.file_name,
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
						file_name: patch.file_name,
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
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/CI",
		move |ctx, mm| {
			Box::pin(patch_editor_ci_page_projection_authorized(
				ctx, mm, case_id, request,
			))
		},
	)
	.await
}

async fn patch_editor_ci_page_projection_authorized(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	request: CaseEditorPagePatchRequest,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let requested_authorities =
		validate_request_projection_context(request.authorities.as_deref())?;
	let fda = request
		.authorities
		.as_deref()
		.unwrap_or_default()
		.iter()
		.any(|authority| authority.eq_ignore_ascii_case("fda"));
	validate_direct_rows("CI", &request.rows, fda)?;
	if !request.rows.is_empty() {
		apply_ci_rows_patch(&ctx, &mm, case_id, &request.rows).await?;
		mark_editor_validation_summary_stale(
			ctx,
			mm,
			case_id,
			requested_authorities.clone(),
		)
		.await?;
	}
	let projection = direct_page_projection_response(
		ctx,
		mm,
		case_id,
		"CI",
		requested_authorities,
		load_editor_ci_data(ctx, mm, case_id).await?,
	)
	.await?;
	Ok((axum::http::StatusCode::OK, Json(projection)))
}

pub async fn patch_editor_rp_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, snapshot, case_id, "RP", request).await
}

pub async fn patch_editor_sd_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, snapshot, case_id, "SD", request).await
}

pub async fn patch_editor_si_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, snapshot, case_id, "SI", request).await
}

pub async fn patch_editor_dm_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, snapshot, case_id, "DM", request).await
}

pub async fn patch_editor_nr_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<CaseEditorPagePatchRequest>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	patch_direct_page_projection(mm, ctx_w, snapshot, case_id, "NR", request).await
}

async fn patch_direct_page_projection(
	mm: ModelManager,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	case_id: Uuid,
	page_id: &'static str,
	request: CaseEditorPagePatchRequest,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("editor/{page_id}"),
		move |ctx, mm| {
			Box::pin(patch_direct_page_projection_authorized(
				ctx, mm, case_id, page_id, request,
			))
		},
	)
	.await
}

async fn patch_direct_page_projection_authorized(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	page_id: &'static str,
	mut request: CaseEditorPagePatchRequest,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let requested_authorities =
		validate_request_projection_context(request.authorities.as_deref())?;
	let fda = request
		.authorities
		.as_deref()
		.unwrap_or_default()
		.iter()
		.any(|authority| authority.eq_ignore_ascii_case("fda"));
	if page_id == "SI"
		&& !request
			.authorities
			.as_deref()
			.unwrap_or_default()
			.iter()
			.any(|authority| authority.eq_ignore_ascii_case("mfds"))
	{
		if let Some(study) = request
			.rows
			.get_mut("studyInformation")
			.and_then(Value::as_object_mut)
		{
			study.remove("studyTypeReactionKr1");
			study.remove("study_type_reaction_kr1");
		}
	}
	validate_direct_rows(page_id, &request.rows, fda)?;

	if !request.rows.is_empty() {
		apply_direct_page_rows_patch(ctx, mm, case_id, page_id, &request.rows)
			.await?;
		mark_editor_validation_summary_stale(
			ctx,
			mm,
			case_id,
			requested_authorities.clone(),
		)
		.await?;
	}

	let data = match page_id {
		"RP" => load_editor_rp_data(ctx, mm, case_id).await?,
		"SD" => load_editor_sd_data(ctx, mm, case_id).await?,
		"SI" => load_editor_si_data(ctx, mm, case_id).await?,
		"DM" => load_editor_dm_data(ctx, mm, case_id).await?,
		"NR" => load_editor_nr_data(ctx, mm, case_id).await?,
		_ => {
			return Err(Error::BadRequest {
				message: format!("unsupported direct page '{page_id}'"),
			})
		}
	};
	let projection = direct_page_projection_response(
		ctx,
		mm,
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
		"SI" => apply_si_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		"DM" => apply_dm_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		"NR" => apply_nr_page_rows_patch(ctx, mm, case_id, page_id, rows).await,
		_ => Err(Error::BadRequest {
			message: format!("unsupported direct page '{page_id}'"),
		}),
	}
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
	let id = uuid_field(source, &["id"]);
	if bool_field(source, &["deleted"]) == Some(true) {
		if let Some(id) = id {
			PrimarySourceBmc::delete(ctx, mm, id).await?;
		}
		return Ok(());
	}
	let update = PrimarySourceForUpdate {
		source_reporter_presave_id: uuid_field(source, &["sourceReporterPresaveId"]),
		reporter_title: string_field(source, &["reporterTitle"]),
		reporter_title_null_flavor: string_field(
			source,
			&["reporterTitleNullFlavor"],
		),
		reporter_given_name: string_field(source, &["reporterGivenName"]),
		reporter_given_name_null_flavor: string_field(
			source,
			&["reporterGivenNameNullFlavor"],
		),
		reporter_middle_name: string_field(source, &["reporterMiddleName"]),
		reporter_middle_name_null_flavor: string_field(
			source,
			&["reporterMiddleNameNullFlavor"],
		),
		reporter_family_name: string_field(source, &["reporterFamilyName"]),
		reporter_family_name_null_flavor: string_field(
			source,
			&["reporterFamilyNameNullFlavor"],
		),
		organization: string_field(source, &["reporterOrganization"]),
		organization_null_flavor: string_field(
			source,
			&["reporterOrganizationNullFlavor"],
		),
		department: string_field(source, &["reporterDepartment"]),
		department_null_flavor: string_field(
			source,
			&["reporterDepartmentNullFlavor"],
		),
		street: string_field(source, &["reporterStreet"]),
		street_null_flavor: string_field(source, &["reporterStreetNullFlavor"]),
		city: string_field(source, &["reporterCity"]),
		city_null_flavor: string_field(source, &["reporterCityNullFlavor"]),
		state: string_field(source, &["reporterState"]),
		state_null_flavor: string_field(source, &["reporterStateNullFlavor"]),
		postcode: string_field(source, &["reporterPostcode"]),
		postcode_null_flavor: string_field(source, &["reporterPostcodeNullFlavor"]),
		telephone: string_field(source, &["reporterTelephone"]),
		telephone_null_flavor: string_field(
			source,
			&["reporterTelephoneNullFlavor"],
		),
		country_code: string_field(source, &["reporterCountry"]),
		email: string_field(source, &["reporterEmail"]),
		email_null_flavor: string_field(source, &["reporterEmailNullFlavor"]),
		qualification: string_field(source, &["qualification"]),
		qualification_null_flavor: string_field(
			source,
			&["qualificationNullFlavor"],
		),
		qualification_kr1: string_field(source, &["qualificationKr1"]),
		primary_source_regulatory: string_field(
			source,
			&["primarySourceForRegulatoryPurposes"],
		),
	};
	if let Some(id) = id {
		PrimarySourceBmc::update(ctx, mm, id, update).await?;
	} else {
		PrimarySourceBmc::create(
			ctx,
			mm,
			PrimarySourceForCreate {
				case_id,
				source_reporter_presave_id: update.source_reporter_presave_id,
				sequence_number: i32_field(source, &["sequenceNumber"]).unwrap_or(1),
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
		&["senderInformation", "receiverInformation"],
	)?;
	if let Some(sender) = optional_row_object(page_id, rows, "senderInformation")? {
		let update = SenderInformationForUpdate {
			source_sender_presave_id: uuid_field(sender, &["sourceSenderPresaveId"]),
			sender_type: string_field(sender, &["senderType"]),
			health_professional_type_kr1: string_field(
				sender,
				&["healthProfessionalTypeKr1"],
			),
			organization_name: string_field(sender, &["organizationName"]),
			department: string_field(sender, &["department"]),
			street_address: string_field(sender, &["streetAddress"]),
			city: string_field(sender, &["city"]),
			state: string_field(sender, &["state"]),
			postcode: string_field(sender, &["postcode"]),
			country_code: string_field(sender, &["countryCode"]),
			person_title: string_field(sender, &["personTitle"]),
			person_given_name: string_field(sender, &["personGivenName"]),
			person_middle_name: string_field(sender, &["personMiddleName"]),
			person_family_name: string_field(sender, &["personFamilyName"]),
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
			receiver_type: string_field(receiver, &["receiverType"]),
			organization_name: string_field(receiver, &["organizationName"]),
			department: string_field(receiver, &["department"]),
			street_address: string_field(receiver, &["streetAddress"]),
			city: string_field(receiver, &["city"]),
			state_province: string_field(receiver, &["stateProvince"]),
			postcode: string_field(receiver, &["postcode"]),
			country_code: string_field(receiver, &["countryCode"]),
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
			source_study_presave_id: uuid_field(study, &["sourceStudyPresaveId"]),
			study_name: string_field(study, &["studyName"]),
			study_name_null_flavor: string_field(study, &["studyNameNullFlavor"]),
			sponsor_study_number: string_field(study, &["sponsorStudyNumber"]),
			sponsor_study_number_null_flavor: string_field(
				study,
				&["sponsorStudyNumberNullFlavor"],
			),
			study_type_reaction: string_field(study, &["studyTypeReaction"]),
			study_type_reaction_kr1: string_field(study, &["studyTypeReactionKr1"]),
			fda_ind_number_occurred: string_field(study, &["fdaIndNumberOccurred"]),
			fda_pre_anda_number_occurred: string_field(
				study,
				&["fdaPreAndaNumberOccurred"],
			),
		};
		if let Some(id) = uuid_field(study, &["id"]) {
			StudyInformationBmc::update(ctx, mm, id, update).await?;
			id
		} else {
			let existing = StudyInformationBmc::list(
				ctx,
				mm,
				Some(vec![StudyInformationFilter {
					case_id: Some(uuid_eq(case_id)),
				}]),
				Some(ListOptions::default()),
			)
			.await?
			.into_iter()
			.min_by_key(|study| study.created_at);
			if let Some(existing) = existing {
				StudyInformationBmc::update(ctx, mm, existing.id, update).await?;
				existing.id
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
			if bool_field(registration, &["deleted"]) == Some(true) {
				if let Some(id) = id {
					StudyRegistrationNumberBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let update = StudyRegistrationNumberForUpdate {
				registration_number: string_field(
					registration,
					&["registrationNumber"],
				),
				registration_number_null_flavor: string_field(
					registration,
					&["registrationNumberNullFlavor"],
				),
				country_code: string_field(registration, &["countryCode"]),
				country_code_null_flavor: string_field(
					registration,
					&["countryCodeNullFlavor"],
				),
				sequence_number: i32_field(registration, &["sequenceNumber"]),
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
		if let Some(value) = study.get("fdaCrossReportedIndNumbers") {
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
				if bool_field(number, &["deleted"]) == Some(true) {
					if let Some(id) = id {
						StudyFdaCrossReportedIndBmc::delete(ctx, mm, id).await?;
					}
					continue;
				}
				let update = StudyFdaCrossReportedIndForUpdate {
					ind_number: string_field(number, &["indNumber"]),
					ind_number_null_flavor: string_field(
						number,
						&["indNumberNullFlavor"],
					),
					sequence_number: i32_field(number, &["sequenceNumber"]),
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
	let patient = optional_row_object(page_id, rows, "patientInformation")?;
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
		_request_path: &str,
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
		_request_path: &str,
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
	fn canonical_string_field(
		row: &Map<String, Value>,
		value_paths: &[&str],
		null_flavor_paths: &[&str],
	) -> (Option<String>, Option<String>) {
		(
			nested_string_field(row, value_paths),
			nested_string_field(row, null_flavor_paths),
		)
	}
	fn string_list_field(
		row: &Map<String, Value>,
		paths: &[&str],
	) -> Option<Vec<String>> {
		let value = value_at_path(row, paths)?;
		if let Some(value) = value.as_str() {
			return Some(vec![value.to_string()]);
		}
		value.as_array().map(|values| {
			values
				.iter()
				.filter_map(Value::as_str)
				.map(ToOwned::to_owned)
				.collect()
		})
	}
	fn null_flavor_field(
		row: &Map<String, Value>,
		paths: &[&str],
	) -> Option<String> {
		nested_string_field(row, paths)
	}
	let patient_id = if let Some(patient) = patient {
		let (patient_initials, patient_initials_null_flavor) =
			canonical_string_field(
				patient,
				&["patientInitials"],
				&["patientInitialsNullFlavor"],
			);
		let birth_date_paths = &["patientBirthDate"];
		let age_paths = &["patientAge.value"];
		let weight_paths = &["patientWeight.value"];
		let height_paths = &["patientHeight.value"];
		let (sex, sex_null_flavor) = canonical_string_field(
			patient,
			&["patientSex"],
			&["patientSexNullFlavor"],
		);
		let race_codes = string_list_field(patient, &["raceCodes", "raceCode"]);
		let race_code_null_flavor =
			null_flavor_field(patient, &["raceCodeNullFlavor"]);
		let (ethnicity_code, ethnicity_code_null_flavor) = canonical_string_field(
			patient,
			&["ethnicityCode"],
			&["ethnicityCodeNullFlavor"],
		);
		let lmp_paths = &["lastMenstrualPeriodDate"];
		let (medical_history_text, medical_history_text_null_flavor) =
			canonical_string_field(
				patient,
				&["medicalHistoryText"],
				&["medicalHistoryTextNullFlavor"],
			);
		let update = PatientInformationForUpdate {
			patient_initials,
			patient_initials_null_flavor,
			birth_date: date_field(
				page_id,
				patient,
				"patientBirthDate",
				birth_date_paths,
			)?,
			birth_date_null_flavor: null_flavor_field(
				patient,
				&["patientBirthDateNullFlavor"],
			),
			age_at_time_of_onset: Some(decimal_field(
				page_id,
				patient,
				"patientAge.value",
				age_paths,
			)?),
			age_unit: nested_string_field(patient, &["patientAge.unit"]),
			gestation_period: decimal_field(
				page_id,
				patient,
				"gestationPeriod.value",
				&["gestationPeriod.value"],
			)?,
			gestation_period_unit: nested_string_field(
				patient,
				&["gestationPeriod.unit"],
			),
			age_group: string_field(patient, &["patientAgeGroup"]),
			weight_kg: Some(decimal_field(
				page_id,
				patient,
				"patientWeight.value",
				weight_paths,
			)?),
			height_cm: Some(decimal_field(
				page_id,
				patient,
				"patientHeight.value",
				height_paths,
			)?),
			sex,
			sex_null_flavor,
			race_codes,
			race_code_null_flavor,
			ethnicity_code,
			ethnicity_code_null_flavor,
			last_menstrual_period_date: date_field(
				page_id,
				patient,
				"lastMenstrualPeriodDate",
				lmp_paths,
			)?,
			last_menstrual_period_date_null_flavor: null_flavor_field(
				patient,
				&["lastMenstrualPeriodDateNullFlavor"],
			),
			medical_history_text,
			medical_history_text_null_flavor,
			concomitant_therapy: boolean_field(patient, &["concomitantTherapies"]),
		};
		match PatientInformationBmc::get_by_case(ctx, mm, case_id).await {
			Ok(entity) => {
				PatientInformationBmc::update_by_case(ctx, mm, case_id, update)
					.await?;
				entity.id
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
						age_at_time_of_onset: update.age_at_time_of_onset.flatten(),
						age_unit: update.age_unit,
						gestation_period: update.gestation_period,
						gestation_period_unit: update.gestation_period_unit,
						age_group: update.age_group,
						weight_kg: update.weight_kg.flatten(),
						height_cm: update.height_cm.flatten(),
						sex: update.sex,
						sex_null_flavor: update.sex_null_flavor,
						race_codes: update.race_codes.unwrap_or_default(),
						race_code_null_flavor: update.race_code_null_flavor,
						ethnicity_code: update.ethnicity_code,
						ethnicity_code_null_flavor: update
							.ethnicity_code_null_flavor,
						last_menstrual_period_date: update
							.last_menstrual_period_date,
						last_menstrual_period_date_null_flavor: update
							.last_menstrual_period_date_null_flavor,
						medical_history_text: update.medical_history_text,
						medical_history_text_null_flavor: update
							.medical_history_text_null_flavor,
						concomitant_therapy: update.concomitant_therapy,
					},
				)
				.await?
			}
			Err(err) => return Err(err.into()),
		}
	} else {
		match PatientInformationBmc::get_by_case(ctx, mm, case_id).await {
			Ok(entity) => entity.id,
			Err(lib_core::model::Error::EntityUuidNotFound { .. }) => {
				return Err(Error::BadRequest {
					message: format!(
						"{page_id}.patientInformation is required before dependent rows"
					),
				});
			}
			Err(err) => return Err(err.into()),
		}
	};

	if let Some(value) = rows.get("medicalHistoryEpisodes") {
		let Some(episodes) = value.as_array() else {
			return Err(Error::BadRequest {
				message: format!(
					"{page_id}.medicalHistoryEpisodes must be an array"
				),
			});
		};
		for (index, value) in episodes.iter().enumerate() {
			let episode = as_object(page_id, "medicalHistoryEpisodes", value)?;
			let id = uuid_field(episode, &["id"]);
			if bool_field(episode, &["deleted"]) == Some(true) {
				if let Some(id) = id {
					MedicalHistoryEpisodeBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let meddra_version = string_field(episode, &["meddraVersion"]);
			let meddra_code = string_field(episode, &["meddraCode"]);
			let start_date = date_field(
				page_id,
				episode,
				"medicalHistoryEpisodes[].startDate",
				&["startDate"],
			)?;
			let start_date_null_flavor =
				null_flavor_field(episode, &["startDateNullFlavor"]);
			let continuing = bool_field(episode, &["continuing"]);
			let continuing_null_flavor =
				null_flavor_field(episode, &["continuingNullFlavor"]);
			let end_date = date_field(
				page_id,
				episode,
				"medicalHistoryEpisodes[].endDate",
				&["endDate"],
			)?;
			let end_date_null_flavor =
				null_flavor_field(episode, &["endDateNullFlavor"]);
			let comments = string_field(episode, &["comments"]);
			let family_history = bool_field(episode, &["familyHistory"]);
			let update = MedicalHistoryEpisodeForUpdate {
				meddra_version,
				meddra_code: meddra_code.clone(),
				start_date,
				start_date_null_flavor: start_date_null_flavor.clone(),
				continuing,
				continuing_null_flavor: continuing_null_flavor.clone(),
				end_date,
				end_date_null_flavor: end_date_null_flavor.clone(),
				comments,
				family_history,
			};
			let id = if let Some(id) = id {
				id
			} else {
				MedicalHistoryEpisodeBmc::create(
					ctx,
					mm,
					MedicalHistoryEpisodeForCreate {
						patient_id,
						sequence_number: i32_field(episode, &["sequenceNumber"])
							.unwrap_or_else(|| {
								i32::try_from(index + 1).unwrap_or(i32::MAX)
							}),
						meddra_code,
						start_date_null_flavor,
						continuing_null_flavor,
						end_date_null_flavor,
					},
				)
				.await?
			};
			MedicalHistoryEpisodeBmc::update(ctx, mm, id, update).await?;
		}
	}

	if let Some(value) = rows.get("patientIdentifiers") {
		let Some(identifier_rows) = value.as_array() else {
			return Err(Error::BadRequest {
				message: format!("{page_id}.patientIdentifiers must be an array"),
			});
		};
		for (index, value) in identifier_rows.iter().enumerate() {
			let identifier = as_object(page_id, "patientIdentifiers", value)?;
			let id = uuid_field(identifier, &["id"]);
			if bool_field(identifier, &["deleted"]) == Some(true) {
				if let Some(id) = id {
					PatientIdentifierBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let identifier_type_code =
				string_field(identifier, &["identifierTypeCode"]);
			let update = PatientIdentifierForUpdate {
				identifier_type_code: identifier_type_code.clone(),
				identifier_value: string_field(identifier, &["identifierValue"]),
				identifier_value_null_flavor: string_field(
					identifier,
					&["identifierValueNullFlavor"],
				),
			};
			if let Some(id) = id {
				PatientIdentifierBmc::update(ctx, mm, id, update).await?;
			} else {
				let identifier_type_code =
					identifier_type_code.ok_or_else(|| Error::BadRequest {
						message: format!(
							"{page_id}.patientIdentifiers.identifierTypeCode is required"
						),
					})?;
				PatientIdentifierBmc::create(
					ctx,
					mm,
					PatientIdentifierForCreate {
						patient_id,
						sequence_number: i32_field(identifier, &["sequenceNumber"])
							.unwrap_or_else(|| {
								i32::try_from(index + 1).unwrap_or(i32::MAX)
							}),
						identifier_type_code,
						identifier_value: update.identifier_value,
						identifier_value_null_flavor: update
							.identifier_value_null_flavor,
					},
				)
				.await?;
			}
		}
	}

	let death_info_row = optional_row_object(page_id, rows, "deathInfo")?;
	let has_death_children =
		rows.contains_key("reportedCauses") || rows.contains_key("autopsyCauses");
	let existing_death_info = PatientDeathInformationBmc::list(
		ctx,
		mm,
		Some(vec![PatientDeathInformationFilter {
			patient_id: Some(uuid_eq(patient_id)),
		}]),
		Some(ListOptions::default()),
	)
	.await?
	.into_iter()
	.next();
	let death_info_id = if let Some(death_info) = death_info_row {
		let update = PatientDeathInformationForUpdate {
			date_of_death: date_field(
				page_id,
				death_info,
				"patientDeath.dateOfDeath",
				&["dateOfDeath"],
			)?,
			date_of_death_null_flavor: null_flavor_field(
				death_info,
				&["dateOfDeathNullFlavor"],
			),
			autopsy_performed: bool_field(death_info, &["autopsyPerformed"]),
			autopsy_performed_null_flavor: null_flavor_field(
				death_info,
				&["autopsyPerformedNullFlavor"],
			),
		};
		if let Some(existing) = existing_death_info {
			PatientDeathInformationBmc::update(ctx, mm, existing.id, update).await?;
			Some(existing.id)
		} else {
			Some(
				PatientDeathInformationBmc::create(
					ctx,
					mm,
					PatientDeathInformationForCreate {
						patient_id,
						date_of_death: update.date_of_death,
						date_of_death_null_flavor: update.date_of_death_null_flavor,
						autopsy_performed: update.autopsy_performed,
						autopsy_performed_null_flavor: update
							.autopsy_performed_null_flavor,
					},
				)
				.await?,
			)
		}
	} else {
		existing_death_info.map(|row| row.id)
	};

	if has_death_children && death_info_id.is_none() {
		return Err(Error::BadRequest {
			message: format!(
				"{page_id}.deathInfo is required before death cause rows"
			),
		});
	}
	let death_info_id = death_info_id.unwrap_or(Uuid::nil());

	if let Some(value) = rows.get("reportedCauses") {
		let Some(causes) = value.as_array() else {
			return Err(Error::BadRequest {
				message: format!("{page_id}.reportedCauses must be an array"),
			});
		};
		for (index, value) in causes.iter().enumerate() {
			let cause = as_object(page_id, "reportedCauses", value)?;
			let id = uuid_field(cause, &["id"]);
			if bool_field(cause, &["deleted"]) == Some(true) {
				if let Some(id) = id {
					ReportedCauseOfDeathBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let update = ReportedCauseOfDeathForUpdate {
				meddra_version: string_field(cause, &["meddraVersion"]),
				meddra_code: string_field(cause, &["meddraCode"]),
				comments: string_field(cause, &["causeText"]),
			};
			if let Some(id) = id {
				ReportedCauseOfDeathBmc::update(ctx, mm, id, update).await?;
			} else {
				ReportedCauseOfDeathBmc::create(
					ctx,
					mm,
					ReportedCauseOfDeathForCreate {
						death_info_id,
						sequence_number: i32_field(cause, &["sequenceNumber"])
							.unwrap_or_else(|| {
								i32::try_from(index + 1).unwrap_or(i32::MAX)
							}),
						meddra_version: update.meddra_version,
						meddra_code: update.meddra_code,
						comments: update.comments,
					},
				)
				.await?;
			}
		}
	}

	if let Some(value) = rows.get("autopsyCauses") {
		let Some(causes) = value.as_array() else {
			return Err(Error::BadRequest {
				message: format!("{page_id}.autopsyCauses must be an array"),
			});
		};
		for (index, value) in causes.iter().enumerate() {
			let cause = as_object(page_id, "autopsyCauses", value)?;
			let id = uuid_field(cause, &["id"]);
			if bool_field(cause, &["deleted"]) == Some(true) {
				if let Some(id) = id {
					AutopsyCauseOfDeathBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let update = AutopsyCauseOfDeathForUpdate {
				meddra_version: string_field(cause, &["meddraVersion"]),
				meddra_code: string_field(cause, &["meddraCode"]),
				comments: string_field(cause, &["causeText"]),
			};
			if let Some(id) = id {
				AutopsyCauseOfDeathBmc::update(ctx, mm, id, update).await?;
			} else {
				AutopsyCauseOfDeathBmc::create(
					ctx,
					mm,
					AutopsyCauseOfDeathForCreate {
						death_info_id,
						sequence_number: i32_field(cause, &["sequenceNumber"])
							.unwrap_or_else(|| {
								i32::try_from(index + 1).unwrap_or(i32::MAX)
							}),
						meddra_version: update.meddra_version,
						meddra_code: update.meddra_code,
						comments: update.comments,
					},
				)
				.await?;
			}
		}
	}

	if let Some(parent) = optional_row_object(page_id, rows, "parentInfo")? {
		let existing = ParentInformationBmc::list(
			ctx,
			mm,
			Some(vec![ParentInformationFilter {
				patient_id: Some(uuid_eq(patient_id)),
				..Default::default()
			}]),
			Some(ListOptions::default()),
		)
		.await?
		.into_iter()
		.next();
		if bool_field(parent, &["deleted"]) == Some(true) {
			if let Some(id) = uuid_field(parent, &["id"])
				.or_else(|| existing.as_ref().map(|row| row.id))
			{
				ParentInformationBmc::delete(ctx, mm, id).await?;
			}
		} else {
			let (parent_identification, parent_identification_null_flavor) =
				canonical_string_field(
					parent,
					&["parentIdentification"],
					&["parentIdentificationNullFlavor"],
				);
			let (sex, sex_null_flavor) = canonical_string_field(
				parent,
				&["parentSex"],
				&["parentSexNullFlavor"],
			);
			let update = ParentInformationForUpdate {
				parent_identification,
				parent_identification_null_flavor,
				parent_birth_date: date_field(
					page_id,
					parent,
					"parentInformation.parentBirthDate",
					&["parentBirthDate"],
				)?,
				parent_birth_date_null_flavor: null_flavor_field(
					parent,
					&["parentBirthDateNullFlavor"],
				),
				parent_age: Some(decimal_field(
					page_id,
					parent,
					"parentInformation.parentAge.value",
					&["parentAge.value"],
				)?),
				parent_age_unit: nested_string_field(parent, &["parentAge.unit"]),
				last_menstrual_period_date: date_field(
					page_id,
					parent,
					"parentInformation.parentLastMenstrualPeriodDate",
					&["parentLastMenstrualPeriodDate"],
				)?,
				last_menstrual_period_date_null_flavor: null_flavor_field(
					parent,
					&["parentLastMenstrualPeriodDateNullFlavor"],
				),
				weight_kg: decimal_field(
					page_id,
					parent,
					"parentInformation.parentWeight.value",
					&["parentWeight.value"],
				)?,
				height_cm: decimal_field(
					page_id,
					parent,
					"parentInformation.parentHeight.value",
					&["parentHeight.value"],
				)?,
				sex,
				sex_null_flavor,
				medical_history_text: string_field(parent, &["medicalHistoryText"]),
			};
			if let Some(existing) = existing {
				ParentInformationBmc::update(ctx, mm, existing.id, update).await?;
			} else {
				ParentInformationBmc::create(
					ctx,
					mm,
					ParentInformationForCreate {
						patient_id,
						parent_identification: update.parent_identification,
						parent_identification_null_flavor: update
							.parent_identification_null_flavor,
						parent_birth_date: update.parent_birth_date,
						parent_birth_date_null_flavor: update
							.parent_birth_date_null_flavor,
						parent_age: update.parent_age.flatten(),
						parent_age_unit: update.parent_age_unit,
						last_menstrual_period_date: update
							.last_menstrual_period_date,
						last_menstrual_period_date_null_flavor: update
							.last_menstrual_period_date_null_flavor,
						weight_kg: update.weight_kg,
						height_cm: update.height_cm,
						sex: update.sex,
						sex_null_flavor: update.sex_null_flavor,
						medical_history_text: update.medical_history_text,
					},
				)
				.await?;
			}
		}
	}

	if let Some(value) = rows.get("parentMedicalHistory") {
		let Some(history_rows) = value.as_array() else {
			return Err(Error::BadRequest {
				message: format!("{page_id}.parentMedicalHistory must be an array"),
			});
		};
		let parent_id = ParentInformationBmc::list(
			ctx,
			mm,
			Some(vec![ParentInformationFilter {
				patient_id: Some(uuid_eq(patient_id)),
				..Default::default()
			}]),
			Some(ListOptions::default()),
		)
		.await?
		.into_iter()
		.next()
		.map(|row| row.id)
		.ok_or_else(|| Error::BadRequest {
			message: format!(
				"{page_id}.parentInfo is required before parent medical history"
			),
		})?;
		for (index, value) in history_rows.iter().enumerate() {
			let history = as_object(page_id, "parentMedicalHistory", value)?;
			let id = uuid_field(history, &["id"]);
			if bool_field(history, &["deleted"]) == Some(true) {
				if let Some(id) = id {
					ParentMedicalHistoryBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let meddra_code = string_field(history, &["meddraCode"]);
			let start_date_null_flavor =
				null_flavor_field(history, &["startDateNullFlavor"]);
			let continuing_null_flavor =
				null_flavor_field(history, &["continuingNullFlavor"]);
			let end_date_null_flavor =
				null_flavor_field(history, &["endDateNullFlavor"]);
			let update = ParentMedicalHistoryForUpdate {
				meddra_version: string_field(history, &["meddraVersion"]),
				meddra_code: meddra_code.clone(),
				start_date: date_field(
					page_id,
					history,
					"parentInformation.medicalHistoryEpisodes[].startDate",
					&["startDate"],
				)?,
				start_date_null_flavor: start_date_null_flavor.clone(),
				continuing: bool_field(history, &["continuing"]),
				continuing_null_flavor: continuing_null_flavor.clone(),
				end_date: date_field(
					page_id,
					history,
					"parentInformation.medicalHistoryEpisodes[].endDate",
					&["endDate"],
				)?,
				end_date_null_flavor: end_date_null_flavor.clone(),
				comments: string_field(history, &["comments"]),
			};
			let id = if let Some(id) = id {
				id
			} else {
				ParentMedicalHistoryBmc::create(
					ctx,
					mm,
					ParentMedicalHistoryForCreate {
						parent_id,
						sequence_number: i32_field(history, &["sequenceNumber"])
							.unwrap_or_else(|| {
								i32::try_from(index + 1).unwrap_or(i32::MAX)
							}),
						meddra_code,
						start_date_null_flavor,
						continuing_null_flavor,
						end_date_null_flavor,
					},
				)
				.await?
			};
			ParentMedicalHistoryBmc::update(ctx, mm, id, update).await?;
		}
	}

	if let Some(value) = rows.get("parentPastDrugs") {
		let Some(drug_rows) = value.as_array() else {
			return Err(Error::BadRequest {
				message: format!("{page_id}.parentPastDrugs must be an array"),
			});
		};
		let parent_id = ParentInformationBmc::list(
			ctx,
			mm,
			Some(vec![ParentInformationFilter {
				patient_id: Some(uuid_eq(patient_id)),
				..Default::default()
			}]),
			Some(ListOptions::default()),
		)
		.await?
		.into_iter()
		.next()
		.map(|row| row.id)
		.ok_or_else(|| Error::BadRequest {
			message: format!(
				"{page_id}.parentInfo is required before parent past drug history"
			),
		})?;
		for (index, value) in drug_rows.iter().enumerate() {
			let drug = as_object(page_id, "parentPastDrugs", value)?;
			let id = uuid_field(drug, &["id"]);
			if bool_field(drug, &["deleted"]) == Some(true) {
				if let Some(id) = id {
					ParentPastDrugHistoryBmc::delete(ctx, mm, id).await?;
				}
				continue;
			}
			let update = ParentPastDrugHistoryForUpdate {
				drug_name: string_field(drug, &["drugName"]),
				mpid: string_field(drug, &["mpid"]),
				mpid_version: string_field(drug, &["mpidVersion"]),
				mfds_medicinal_product_version: string_field(
					drug,
					&["mfdsMedicinalProductVersion"],
				),
				mfds_medicinal_product_id: string_field(
					drug,
					&["mfdsMedicinalProductId"],
				),
				phpid: string_field(drug, &["phpid"]),
				phpid_version: string_field(drug, &["phpidVersion"]),
				start_date: date_field(
					page_id,
					drug,
					"parentInformation.pastDrugHistory[].startDate",
					&["startDate"],
				)?,
				start_date_null_flavor: null_flavor_field(
					drug,
					&["startDateNullFlavor"],
				),
				end_date: date_field(
					page_id,
					drug,
					"parentInformation.pastDrugHistory[].endDate",
					&["endDate"],
				)?,
				end_date_null_flavor: null_flavor_field(
					drug,
					&["endDateNullFlavor"],
				),
				indication_meddra_version: string_field(
					drug,
					&["indicationMeddraVersion"],
				),
				indication_meddra_code: string_field(
					drug,
					&["indicationMeddraCode"],
				),
				reaction_meddra_version: string_field(
					drug,
					&["reactionMeddraVersion"],
				),
				reaction_meddra_code: string_field(drug, &["reactionMeddraCode"]),
			};
			if let Some(id) = id {
				ParentPastDrugHistoryBmc::update(ctx, mm, id, update).await?;
			} else {
				ParentPastDrugHistoryBmc::create(
					ctx,
					mm,
					ParentPastDrugHistoryForCreate {
						parent_id,
						sequence_number: i32_field(drug, &["sequenceNumber"])
							.unwrap_or_else(|| {
								i32::try_from(index + 1).unwrap_or(i32::MAX)
							}),
						drug_name: update.drug_name,
						mpid: update.mpid,
						mpid_version: update.mpid_version,
						mfds_medicinal_product_version: update
							.mfds_medicinal_product_version,
						mfds_medicinal_product_id: update.mfds_medicinal_product_id,
						phpid: update.phpid,
						phpid_version: update.phpid_version,
						start_date: update.start_date,
						start_date_null_flavor: update.start_date_null_flavor,
						end_date: update.end_date,
						end_date_null_flavor: update.end_date_null_flavor,
						indication_meddra_version: update.indication_meddra_version,
						indication_meddra_code: update.indication_meddra_code,
						reaction_meddra_version: update.reaction_meddra_version,
						reaction_meddra_code: update.reaction_meddra_code,
					},
				)
				.await?;
			}
		}
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
	if let Some(narrative) = optional_row_object(page_id, rows, "narrative")? {
		let case_narrative = string_field(narrative, &["caseNarrative"]);
		let update = NarrativeInformationForUpdate {
			source_narrative_presave_id: uuid_field(
				narrative,
				&["sourceNarrativePresaveId"],
			),
			case_narrative: case_narrative.clone(),
			reporter_comments: string_field(narrative, &["reporterComments"]),
			sender_comments: string_field(narrative, &["senderComments"]),
			additional_information: string_field(
				narrative,
				&["additionalInformation"],
			),
		};
		match NarrativeInformationBmc::get_by_case_optional(ctx, mm, case_id).await?
		{
			Some(_) => {
				NarrativeInformationBmc::update_by_case(ctx, mm, case_id, update)
					.await?
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
						source_narrative_presave_id: update
							.source_narrative_presave_id,
						case_narrative,
						reporter_comments: update.reporter_comments,
						sender_comments: update.sender_comments,
						additional_information: update.additional_information,
					},
				)
				.await?;
			}
		}
	}

	let has_nested_rows = rows.contains_key("senderDiagnoses")
		|| rows.contains_key("caseSummaryInformation");
	let Some(narrative) =
		NarrativeInformationBmc::get_by_case_optional(ctx, mm, case_id).await?
	else {
		if has_nested_rows {
			return Err(Error::BadRequest {
				message: format!(
					"{page_id} nested rows require an existing narrative"
				),
			});
		}
		return Ok(());
	};

	if let Some(value) = rows.get("senderDiagnoses") {
		let diagnoses = value.as_array().ok_or_else(|| Error::BadRequest {
			message: format!("{page_id}.senderDiagnoses must be an array"),
		})?;
		for (index, value) in diagnoses.iter().enumerate() {
			let diagnosis = as_object(page_id, "senderDiagnoses", value)?;
			let id = uuid_field(diagnosis, &["id"]);
			let deleted = bool_field(diagnosis, &["deleted"]).unwrap_or(false);
			if let Some(id) = id {
				let persisted = SenderDiagnosisBmc::get(ctx, mm, id).await?;
				if persisted.narrative_id != narrative.id {
					return Err(Error::BadRequest {
						message: format!(
							"{page_id}.senderDiagnoses[{index}].id does not belong to the current narrative"
						),
					});
				}
				if deleted {
					SenderDiagnosisBmc::delete(ctx, mm, id).await?;
				} else {
					SenderDiagnosisBmc::update(
						ctx,
						mm,
						id,
						SenderDiagnosisForUpdate {
							diagnosis_meddra_version: string_field(
								diagnosis,
								&["diagnosisMeddraVersion"],
							),
							diagnosis_meddra_code: string_field(
								diagnosis,
								&["diagnosisMeddraCode"],
							),
						},
					)
					.await?;
				}
			} else if !deleted {
				SenderDiagnosisBmc::create(
					ctx,
					mm,
					SenderDiagnosisForCreate {
						narrative_id: narrative.id,
						sequence_number: i32_field(diagnosis, &["sequenceNumber"])
							.unwrap_or((index + 1) as i32),
						diagnosis_meddra_version: string_field(
							diagnosis,
							&["diagnosisMeddraVersion"],
						),
						diagnosis_meddra_code: string_field(
							diagnosis,
							&["diagnosisMeddraCode"],
						),
					},
				)
				.await?;
			}
		}
	}

	if let Some(value) = rows.get("caseSummaryInformation") {
		let summaries = value.as_array().ok_or_else(|| Error::BadRequest {
			message: format!("{page_id}.caseSummaryInformation must be an array"),
		})?;
		for (index, value) in summaries.iter().enumerate() {
			let summary = as_object(page_id, "caseSummaryInformation", value)?;
			let id = uuid_field(summary, &["id"]);
			let deleted = bool_field(summary, &["deleted"]).unwrap_or(false);
			if let Some(id) = id {
				let persisted = CaseSummaryInformationBmc::get(ctx, mm, id).await?;
				if persisted.narrative_id != narrative.id {
					return Err(Error::BadRequest {
						message: format!(
							"{page_id}.caseSummaryInformation[{index}].id does not belong to the current narrative"
						),
					});
				}
				if deleted {
					CaseSummaryInformationBmc::delete(ctx, mm, id).await?;
				} else {
					CaseSummaryInformationBmc::update(
						ctx,
						mm,
						id,
						CaseSummaryInformationForUpdate {
							language_code: string_field(summary, &["languageCode"]),
							summary_text: string_field(summary, &["summaryText"]),
						},
					)
					.await?;
				}
			} else if !deleted {
				CaseSummaryInformationBmc::create(
					ctx,
					mm,
					CaseSummaryInformationForCreate {
						narrative_id: narrative.id,
						sequence_number: i32_field(summary, &["sequenceNumber"])
							.unwrap_or((index + 1) as i32),
						language_code: string_field(summary, &["languageCode"]),
						summary_text: string_field(summary, &["summaryText"]),
					},
				)
				.await?;
			}
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
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/RP",
		move |ctx, mm| {
			Box::pin(async move {
				Ok(direct_section_response(
					case_id,
					load_editor_rp_data(ctx, mm, case_id).await?,
				))
			})
		},
	)
	.await
}

direct_page_projection_handler!(
	get_editor_rp_page_projection,
	"RP",
	load_editor_rp_data,
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

	Ok(json!({
		"senderInformation": sender.map(|row| json!({
			"id": row.id,
			"sourceSenderPresaveId": row.source_sender_presave_id,
			"senderType": row.sender_type,
			"healthProfessionalTypeKr1": row.health_professional_type_kr1,
			"organizationName": row.organization_name,
			"department": row.department,
			"streetAddress": row.street_address,
			"city": row.city,
			"state": row.state,
			"postcode": row.postcode,
			"countryCode": row.country_code,
			"personTitle": row.person_title,
			"personGivenName": row.person_given_name,
			"personMiddleName": row.person_middle_name,
			"personFamilyName": row.person_family_name,
			"telephone": row.telephone,
			"fax": row.fax,
			"email": row.email,
		})),
		"receiverInformation": receiver.map(|row| json!({
			"id": row.id,
			"receiverType": row.receiver_type,
			"organizationName": row.organization_name,
			"department": row.department,
			"streetAddress": row.street_address,
			"city": row.city,
			"stateProvince": row.state_province,
			"postcode": row.postcode,
			"countryCode": row.country_code,
			"telephone": row.telephone,
			"fax": row.fax,
			"email": row.email,
		})),
	}))
}

pub async fn get_editor_sd(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/SD",
		move |ctx, mm| {
			Box::pin(async move {
				Ok(direct_section_response(
					case_id,
					load_editor_sd_data(ctx, mm, case_id).await?,
				))
			})
		},
	)
	.await
}

direct_page_projection_handler!(
	get_editor_sd_page_projection,
	"SD",
	load_editor_sd_data,
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
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/SI",
		move |ctx, mm| {
			Box::pin(async move {
				Ok(direct_section_response(
					case_id,
					load_editor_si_data(ctx, mm, case_id).await?,
				))
			})
		},
	)
	.await
}

direct_page_projection_handler!(
	get_editor_si_page_projection,
	"SI",
	load_editor_si_data,
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
	let medical_history_episodes = medical_history_episodes
		.into_iter()
		.map(|episode| {
			let mut value = json!(episode);
			if let Value::Object(ref mut map) = value {
				map.insert(
					"start_date".to_string(),
					json!(ci_date(episode.start_date)),
				);
				map.insert("end_date".to_string(), json!(ci_date(episode.end_date)));
			}
			value
		})
		.collect::<Vec<_>>();
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
		let medical_history = medical_history
			.into_iter()
			.map(|history| {
				let mut value = json!(history);
				if let Value::Object(ref mut map) = value {
					map.insert(
						"start_date".to_string(),
						json!(ci_date(history.start_date)),
					);
					map.insert(
						"end_date".to_string(),
						json!(ci_date(history.end_date)),
					);
				}
				value
			})
			.collect::<Vec<_>>();
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
		let past_drug_history = past_drug_history
			.into_iter()
			.map(|drug| {
				let mut value = json!(drug);
				if let Value::Object(ref mut map) = value {
					map.insert(
						"start_date".to_string(),
						json!(ci_date(drug.start_date)),
					);
					map.insert(
						"end_date".to_string(),
						json!(ci_date(drug.end_date)),
					);
				}
				value
			})
			.collect::<Vec<_>>();
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
	let death_info = death_information.into_iter().next().map(|death_info| {
		let mut value = json!(death_info);
		if let Value::Object(ref mut map) = value {
			map.insert(
				"date_of_death".to_string(),
				json!(ci_date(death_info.date_of_death)),
			);
		}
		value
	});
	let parent_info = parent_information_rows.into_iter().next().map(|parent| {
		let mut value = json!(parent);
		if let Value::Object(ref mut map) = value {
			map.insert(
				"parent_birth_date".to_string(),
				json!(ci_date(parent.parent_birth_date)),
			);
			map.insert(
				"last_menstrual_period_date".to_string(),
				json!(ci_date(parent.last_menstrual_period_date)),
			);
		}
		value
	});
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
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/DM",
		move |ctx, mm| {
			Box::pin(async move {
				Ok(direct_section_response(
					case_id,
					load_editor_dm_data(ctx, mm, case_id).await?,
				))
			})
		},
	)
	.await
}

direct_page_projection_handler!(
	get_editor_dm_page_projection,
	"DM",
	load_editor_dm_data,
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
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorDirectSectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/NR",
		move |ctx, mm| {
			Box::pin(async move {
				Ok(direct_section_response(
					case_id,
					load_editor_nr_data(ctx, mm, case_id).await?,
				))
			})
		},
	)
	.await
}

direct_page_projection_handler!(
	get_editor_nr_page_projection,
	"NR",
	load_editor_nr_data,
);
