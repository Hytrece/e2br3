use crate::runtime_settings::{self, normalize_appendices};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{NaiveDate, Utc};
use lib_core::authorization::eligible_action_ids;
use lib_core::ctx::{
	canonical_role, Ctx, ROLE_SPONSOR_ADMIN_COMPANY, ROLE_SPONSOR_ADMIN_CRO,
	ROLE_USER,
};
use lib_core::model::admin_settings::AdminSettingsBmc;
use lib_core::model::ModelManager;
use lib_rest_core::{
	notice_read_allowed, with_authorized_notice_update,
	with_authorized_settings_read, with_authorized_settings_update, Error, Result,
};
use lib_web::middleware::mw_auth::CtxW;
use lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use uuid::Uuid;

const SETTINGS_KEY: &str = "system";

fn notice_update_allowed(
	snapshot: &lib_core::authorization::RequestAuthorizationSnapshot,
) -> bool {
	eligible_action_ids(snapshot)
		.iter()
		.any(|action| action.as_str() == "notice.update")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardNoticePayload {
	pub id: Option<String>,
	pub title: String,
	pub body: Option<String>,
	pub effective_date: Option<String>,
	pub expire_date: Option<String>,
	pub writer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminNoticesPayload {
	pub notices: Vec<DashboardNoticePayload>,
	pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatusConfigPayload {
	pub name: String,
	pub editable: bool,
	pub description: Option<String>,
	pub allowed_roles: Option<Vec<String>>,
	pub due_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfigPayload {
	pub statuses: Option<Vec<WorkflowStatusConfigPayload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDateUpdatePayload {
	pub date_of_creation: Option<bool>,
	pub most_recent_info_date: Option<bool>,
	pub report_first_received_date: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSettingsPayload {
	pub timezone: Option<String>,
	pub meddra_language: Option<String>,
	pub meddra_version: Option<String>,
	pub idf_version: Option<String>,
	pub company_logo: Option<String>,
	pub orientation: Option<String>,
	pub data_ordering: Option<String>,
	pub upload_excel_template_without_element_label: Option<bool>,
	pub notation: Option<bool>,
	pub apply_comments_on_exported_xml: Option<bool>,
	pub apply_sender_info_to_imported_cases: Option<bool>,
	pub import_date_update: Option<ImportDateUpdatePayload>,
	pub appendices: Option<Vec<String>>,
	pub case_number_prefix: Option<String>,
	pub case_number_setting: Option<String>,
	pub case_number_identifier: Option<String>,
	pub case_number_padding: Option<i32>,
	pub case_number_sequence_condition: Option<String>,
	pub case_number_format_fields: Option<Vec<String>>,
	pub workflow_enabled: Option<bool>,
	pub workflow: Option<WorkflowConfigPayload>,
	pub idle_session_minutes: Option<i32>,
	pub session_warning_minutes: Option<i32>,
	pub notices: Option<Vec<DashboardNoticePayload>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeSettingsPayload {
	pub timezone: String,
	pub meddra_language: Option<String>,
	pub meddra_version: Option<String>,
	pub orientation: String,
	pub data_ordering: String,
	pub notation: bool,
	pub apply_sender_info_to_imported_cases: bool,
	pub import_date_update: ImportDateUpdatePayload,
	pub appendices: Vec<String>,
	pub idle_session_minutes: i32,
	pub session_warning_minutes: i32,
	pub notices: Vec<DashboardNoticePayload>,
	pub notices_revision: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminNoticesUpdateBody {
	pub notices: Vec<DashboardNoticePayload>,
	pub revision: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminSettingsUpdateBody {
	pub timezone: Option<String>,
	pub meddra_language: Option<String>,
	pub meddra_version: Option<String>,
	pub idf_version: Option<String>,
	pub company_logo: Option<String>,
	pub orientation: Option<String>,
	pub data_ordering: Option<String>,
	pub upload_excel_template_without_element_label: Option<bool>,
	pub notation: Option<bool>,
	pub apply_comments_on_exported_xml: Option<bool>,
	pub apply_sender_info_to_imported_cases: Option<bool>,
	pub import_date_update: Option<ImportDateUpdatePayload>,
	pub appendices: Option<Vec<String>>,
	pub case_number_prefix: Option<String>,
	pub case_number_setting: Option<String>,
	pub case_number_identifier: Option<String>,
	pub case_number_padding: Option<i32>,
	pub case_number_sequence_condition: Option<String>,
	pub case_number_format_fields: Option<Vec<String>>,
	pub workflow_enabled: Option<bool>,
	pub workflow: Option<WorkflowConfigPayload>,
	pub idle_session_minutes: Option<i32>,
	pub session_warning_minutes: Option<i32>,
}

fn default_settings() -> AdminSettingsPayload {
	AdminSettingsPayload {
		timezone: Some(runtime_settings::DEFAULT_TIMEZONE.to_string()),
		meddra_language: Some("English".to_string()),
		meddra_version: Some(String::new()),
		idf_version: Some(String::new()),
		company_logo: Some(String::new()),
		orientation: Some("Landscape".to_string()),
		data_ordering: Some("Primary data will appear first".to_string()),
		upload_excel_template_without_element_label: Some(false),
		notation: Some(runtime_settings::DEFAULT_NOTATION),
		apply_comments_on_exported_xml: Some(false),
		apply_sender_info_to_imported_cases: Some(false),
		import_date_update: Some(ImportDateUpdatePayload {
			date_of_creation: Some(false),
			most_recent_info_date: Some(false),
			report_first_received_date: Some(false),
		}),
		appendices: Some(vec!["ICH".to_string()]),
		case_number_prefix: Some("ICSR".to_string()),
		case_number_setting: Some(String::new()),
		case_number_identifier: Some(String::new()),
		case_number_padding: Some(6),
		case_number_sequence_condition: Some(String::new()),
		case_number_format_fields: Some(Vec::new()),
		workflow_enabled: Some(false),
		workflow: Some(default_workflow_config()),
		idle_session_minutes: Some(60),
		session_warning_minutes: Some(5),
		notices: Some(Vec::new()),
	}
}

async fn load_notices(
	ctx: &Ctx,
	mm: &ModelManager,
) -> Result<Vec<DashboardNoticePayload>> {
	let values = AdminSettingsBmc::list_dashboard_notices(ctx, mm)
		.await
		.map_err(Error::Model)?;
	let notices = values
		.into_iter()
		.map(serde_json::from_value)
		.collect::<std::result::Result<Vec<_>, _>>()?;
	Ok(notices)
}

async fn current_user_email(
	ctx: &Ctx,
	mm: &ModelManager,
	user_id: Uuid,
) -> Result<String> {
	let user: lib_core::model::user::User =
		lib_core::model::user::UserBmc::get(ctx, mm, user_id)
			.await
			.map_err(|err| Error::BadRequest {
				message: format!("failed to resolve current user email: {err}"),
			})?;
	Ok(user.email)
}

fn normalize_notice_date(
	value: Option<String>,
	field: &str,
) -> Result<Option<String>> {
	let Some(value) = value.map(|value| value.trim().to_string()) else {
		return Ok(None);
	};
	if value.is_empty() {
		return Ok(None);
	}
	let parsed = NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_| {
		Error::BadRequest {
			message: format!("{field} must be an ISO date in YYYY-MM-DD format"),
		}
	})?;
	if parsed.format("%Y-%m-%d").to_string() != value {
		return Err(Error::BadRequest {
			message: format!("{field} must be an ISO date in YYYY-MM-DD format"),
		});
	}
	Ok(Some(value))
}

fn import_date_update_is_supported(value: &ImportDateUpdatePayload) -> bool {
	matches!(
		(
			value.date_of_creation.unwrap_or(false),
			value.most_recent_info_date.unwrap_or(false),
			value.report_first_received_date.unwrap_or(false),
		),
		(false, false, false)
			| (true, false, true)
			| (true, true, false)
			| (true, true, true)
	)
}

fn normalize_notices(
	notices: Vec<DashboardNoticePayload>,
	writer: String,
) -> Result<Vec<DashboardNoticePayload>> {
	let mut seen_ids = HashSet::new();
	let mut normalized = Vec::new();
	for (index, notice) in notices.into_iter().enumerate() {
		let title = notice.title.trim().to_string();
		let body = notice.body.unwrap_or_default().trim().to_string();
		if title.is_empty() && body.is_empty() {
			continue;
		}
		let id = notice
			.id
			.map(|value| value.trim().to_string())
			.filter(|value| !value.is_empty())
			.unwrap_or_else(|| format!("notice-{}", index + 1));
		if !seen_ids.insert(id.clone()) {
			return Err(Error::BadRequest {
				message: format!("duplicate notice id '{id}'"),
			});
		}
		let effective_date =
			normalize_notice_date(notice.effective_date, "effective_date")?;
		let expire_date = normalize_notice_date(notice.expire_date, "expire_date")?;
		if let (Some(effective_date), Some(expire_date)) =
			(effective_date.as_deref(), expire_date.as_deref())
		{
			if effective_date > expire_date {
				return Err(Error::BadRequest {
					message: "effective_date must be on or before expire_date"
						.to_string(),
				});
			}
		}
		normalized.push(DashboardNoticePayload {
			id: Some(id),
			title,
			body: if body.is_empty() { None } else { Some(body) },
			effective_date,
			expire_date,
			writer: Some(writer.clone()),
		});
	}
	Ok(normalized)
}

fn default_workflow_config() -> WorkflowConfigPayload {
	WorkflowConfigPayload {
		statuses: Some(vec![
			WorkflowStatusConfigPayload {
				name: "Saved".to_string(),
				editable: true,
				description: Some("Default authoring state".to_string()),
				due_days: Some(0),
				allowed_roles: Some(vec![ROLE_USER.to_string()]),
			},
			WorkflowStatusConfigPayload {
				name: "To be reviewed".to_string(),
				editable: false,
				description: Some("Pending internal review".to_string()),
				due_days: Some(0),
				allowed_roles: Some(vec![ROLE_USER.to_string()]),
			},
			WorkflowStatusConfigPayload {
				name: "Internal review completed".to_string(),
				editable: false,
				description: Some("QCed and routed onward".to_string()),
				due_days: Some(0),
				allowed_roles: Some(vec![ROLE_USER.to_string()]),
			},
			WorkflowStatusConfigPayload {
				name: "Finalized".to_string(),
				editable: false,
				description: Some("Final workflow state".to_string()),
				due_days: Some(0),
				allowed_roles: Some(vec![
					ROLE_SPONSOR_ADMIN_CRO.to_string(),
					ROLE_SPONSOR_ADMIN_COMPANY.to_string(),
				]),
			},
		]),
	}
}

async fn normalize_workflow_config(
	ctx: &Ctx,
	mm: &ModelManager,
	workflow: Option<WorkflowConfigPayload>,
) -> Result<WorkflowConfigPayload> {
	let known_roles = AdminSettingsBmc::known_workflow_roles(ctx, mm)
		.await
		.map_err(Error::Model)?;
	let mut statuses = workflow
		.and_then(|config| config.statuses)
		.unwrap_or_default()
		.into_iter()
		.filter_map(|status| {
			let name = status.name.trim().to_string();
			if name.is_empty() {
				None
			} else {
				Some(WorkflowStatusConfigPayload {
					name,
					editable: status.editable,
					description: status.description.map(|v| v.trim().to_string()),
					due_days: status.due_days,
					allowed_roles: status.allowed_roles.map(|roles| {
						roles
							.into_iter()
							.map(|role| canonical_role(role.trim()))
							.filter(|role| !role.is_empty())
							.collect()
					}),
				})
			}
		})
		.collect::<Vec<_>>();

	if statuses.is_empty() {
		statuses = default_workflow_config().statuses.unwrap_or_default();
	}

	let mut seen = HashSet::new();
	for status in &statuses {
		let key = status.name.to_ascii_lowercase();
		if !seen.insert(key) {
			return Err(Error::BadRequest {
				message: format!("duplicate workflow status '{}'", status.name),
			});
		}
	}

	if !statuses.iter().any(|status| status.editable) {
		return Err(Error::BadRequest {
			message: "workflow must define at least one editable status".to_string(),
		});
	}

	if !statuses
		.iter()
		.any(|status| status.name.eq_ignore_ascii_case("Saved"))
	{
		statuses.insert(
			0,
			WorkflowStatusConfigPayload {
				name: "Saved".to_string(),
				editable: true,
				description: Some("Default authoring state".to_string()),
				due_days: Some(0),
				allowed_roles: Some(vec![ROLE_USER.to_string()]),
			},
		);
	}

	for status in &statuses {
		if status.due_days.unwrap_or(0) < 0 {
			return Err(Error::BadRequest {
				message: format!(
					"workflow status '{}' due_days must be zero or greater",
					status.name
				),
			});
		}
		for role in status.allowed_roles.as_deref().unwrap_or(&[]) {
			if !known_roles.contains(role) {
				return Err(Error::BadRequest {
					message: format!(
						"workflow status '{}' references unknown role '{}'",
						status.name, role
					),
				});
			}
		}
	}

	let configured_statuses = statuses
		.iter()
		.map(|status| status.name.to_ascii_lowercase())
		.collect::<HashSet<_>>();
	for status in AdminSettingsBmc::list_workflow_statuses_in_use(ctx, mm)
		.await
		.map_err(Error::Model)?
	{
		if !configured_statuses.contains(&status.to_ascii_lowercase()) {
			return Err(Error::BadRequest {
				message: format!(
					"cannot remove workflow status '{status}'; it is referenced by existing cases"
				),
			});
		}
	}

	Ok(WorkflowConfigPayload {
		statuses: Some(statuses),
	})
}

async fn payload_to_value(
	ctx: &Ctx,
	mm: &ModelManager,
	existing: Option<&Value>,
	payload: &AdminSettingsUpdateBody,
) -> Result<Value> {
	let mut merged = existing
		.cloned()
		.unwrap_or(serde_json::to_value(default_settings())?);
	let object = merged.as_object_mut().ok_or_else(|| Error::BadRequest {
		message: "stored admin settings must be a JSON object".to_string(),
	})?;
	object.remove("notices");
	let existing_timezone = object
		.get("timezone")
		.and_then(Value::as_str)
		.unwrap_or(runtime_settings::DEFAULT_TIMEZONE);
	let existing_timezone = runtime_settings::validate_timezone(existing_timezone)
		.ok_or_else(|| Error::BadRequest {
		message: "stored timezone must be a valid IANA timezone".to_string(),
	})?;
	object.insert("timezone".to_string(), json!(existing_timezone));

	if let Some(timezone) = payload.timezone.as_deref() {
		let timezone =
			runtime_settings::validate_timezone(timezone).ok_or_else(|| {
				Error::BadRequest {
					message: "timezone must be a valid IANA timezone".to_string(),
				}
			})?;
		object.insert("timezone".to_string(), json!(timezone));
	}

	if let Some(orientation) = payload.orientation.as_deref() {
		let orientation = if orientation.eq_ignore_ascii_case("portrait") {
			"Portrait"
		} else if orientation.eq_ignore_ascii_case("landscape") {
			"Landscape"
		} else {
			return Err(Error::BadRequest {
				message: "orientation must be Portrait or Landscape".to_string(),
			});
		};
		object.insert("orientation".to_string(), json!(orientation));
	}

	fn set_if_present<T: Serialize>(
		object: &mut Map<String, Value>,
		key: &str,
		value: Option<&T>,
	) -> Result<()> {
		if let Some(value) = value {
			object.insert(key.to_string(), serde_json::to_value(value)?);
		}
		Ok(())
	}

	set_if_present(object, "meddra_language", payload.meddra_language.as_ref())?;
	set_if_present(object, "meddra_version", payload.meddra_version.as_ref())?;
	set_if_present(object, "idf_version", payload.idf_version.as_ref())?;
	set_if_present(object, "company_logo", payload.company_logo.as_ref())?;
	set_if_present(object, "data_ordering", payload.data_ordering.as_ref())?;
	set_if_present(
		object,
		"upload_excel_template_without_element_label",
		payload.upload_excel_template_without_element_label.as_ref(),
	)?;
	set_if_present(object, "notation", payload.notation.as_ref())?;
	set_if_present(
		object,
		"apply_comments_on_exported_xml",
		payload.apply_comments_on_exported_xml.as_ref(),
	)?;
	set_if_present(
		object,
		"apply_sender_info_to_imported_cases",
		payload.apply_sender_info_to_imported_cases.as_ref(),
	)?;
	set_if_present(
		object,
		"case_number_prefix",
		payload.case_number_prefix.as_ref(),
	)?;
	set_if_present(
		object,
		"case_number_setting",
		payload.case_number_setting.as_ref(),
	)?;
	set_if_present(
		object,
		"case_number_identifier",
		payload.case_number_identifier.as_ref(),
	)?;
	set_if_present(
		object,
		"case_number_padding",
		payload.case_number_padding.as_ref(),
	)?;
	set_if_present(
		object,
		"case_number_sequence_condition",
		payload.case_number_sequence_condition.as_ref(),
	)?;
	set_if_present(
		object,
		"case_number_format_fields",
		payload.case_number_format_fields.as_ref(),
	)?;
	set_if_present(
		object,
		"workflow_enabled",
		payload.workflow_enabled.as_ref(),
	)?;

	let existing_idle = object
		.get("idle_session_minutes")
		.and_then(Value::as_i64)
		.and_then(|value| i32::try_from(value).ok())
		.unwrap_or(60);
	let existing_warning = object
		.get("session_warning_minutes")
		.and_then(Value::as_i64)
		.and_then(|value| i32::try_from(value).ok())
		.unwrap_or(5);
	let idle_session_minutes = payload.idle_session_minutes.unwrap_or(existing_idle);
	let session_warning_minutes =
		payload.session_warning_minutes.unwrap_or(existing_warning);
	if idle_session_minutes < 5 {
		return Err(Error::BadRequest {
			message: "idle_session_minutes must be at least 5".to_string(),
		});
	}
	if session_warning_minutes < 1 {
		return Err(Error::BadRequest {
			message: "session_warning_minutes must be at least 1".to_string(),
		});
	}
	if session_warning_minutes >= idle_session_minutes {
		return Err(Error::BadRequest {
			message:
				"session_warning_minutes must be less than idle_session_minutes"
					.to_string(),
		});
	}
	let case_number_padding = payload.case_number_padding.unwrap_or_else(|| {
		object
			.get("case_number_padding")
			.and_then(Value::as_i64)
			.and_then(|value| i32::try_from(value).ok())
			.unwrap_or(6)
	});
	if case_number_padding < 0 {
		return Err(Error::BadRequest {
			message: "case_number_padding must be zero or greater".to_string(),
		});
	}
	set_if_present(object, "idle_session_minutes", Some(&idle_session_minutes))?;
	set_if_present(
		object,
		"session_warning_minutes",
		Some(&session_warning_minutes),
	)?;

	if let Some(appendices) = payload.appendices.as_deref() {
		let appendices = normalize_appendices(Some(appendices));
		if appendices.is_empty() {
			return Err(Error::BadRequest {
				message: "appendices must include at least one supported authority"
					.to_string(),
			});
		}
		object.insert("appendices".to_string(), json!(appendices));
	}

	let mut import_date_update = object
		.get("import_date_update")
		.and_then(Value::as_object)
		.cloned()
		.unwrap_or_default();
	if let Some(import_date) = payload.import_date_update.as_ref() {
		set_if_present(
			&mut import_date_update,
			"date_of_creation",
			import_date.date_of_creation.as_ref(),
		)?;
		set_if_present(
			&mut import_date_update,
			"most_recent_info_date",
			import_date.most_recent_info_date.as_ref(),
		)?;
		set_if_present(
			&mut import_date_update,
			"report_first_received_date",
			import_date.report_first_received_date.as_ref(),
		)?;
	}
	let import_date_update = serde_json::from_value::<ImportDateUpdatePayload>(
		Value::Object(import_date_update.clone()),
	)
	.map_err(|_| Error::BadRequest {
		message: "import_date_update contains invalid boolean values".to_string(),
	})?;
	if !import_date_update_is_supported(&import_date_update) {
		return Err(Error::BadRequest {
			message: "import_date_update must be one of the four supported states"
				.to_string(),
		});
	}
	object.insert(
		"import_date_update".to_string(),
		serde_json::to_value(import_date_update)?,
	);

	if let Some(workflow) = payload.workflow.clone() {
		if workflow.statuses.is_some() {
			let workflow =
				normalize_workflow_config(ctx, mm, Some(workflow)).await?;
			object.insert("workflow".to_string(), serde_json::to_value(workflow)?);
		}
	}

	Ok(merged)
}

async fn load_admin_settings_payload(
	ctx: &Ctx,
	mm: &ModelManager,
) -> Result<AdminSettingsPayload> {
	let value = AdminSettingsBmc::get(ctx, mm, SETTINGS_KEY)
		.await
		.map_err(Error::Model)?;
	if let Some(value) = value {
		let mut payload = serde_json::from_value::<AdminSettingsPayload>(value)?;
		payload.appendices =
			Some(normalize_appendices(payload.appendices.as_deref()));
		payload.notices = Some(load_notices(ctx, mm).await?);
		return Ok(payload);
	}
	let mut payload = default_settings();
	payload.appendices = Some(normalize_appendices(payload.appendices.as_deref()));
	payload.notices = Some(load_notices(ctx, mm).await?);
	Ok(payload)
}

fn active_notices(
	notices: Vec<DashboardNoticePayload>,
	timezone: &str,
) -> Result<Vec<DashboardNoticePayload>> {
	let timezone =
		timezone
			.parse::<chrono_tz::Tz>()
			.map_err(|_| Error::BadRequest {
				message: "stored timezone must be a valid IANA timezone".to_string(),
			})?;
	let today = Utc::now().with_timezone(&timezone).date_naive();
	Ok(notices
		.into_iter()
		.filter(|notice| {
			let effective = notice
				.effective_date
				.as_deref()
				.map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d"));
			let expire = notice
				.expire_date
				.as_deref()
				.map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d"));
			let effective_ok = match effective {
				None => true,
				Some(Ok(value)) => value <= today,
				Some(Err(_)) => false,
			};
			let expire_ok = match expire {
				None => true,
				Some(Ok(value)) => value >= today,
				Some(Err(_)) => false,
			};
			effective_ok && expire_ok
		})
		.collect())
}

fn runtime_settings_payload(
	payload: AdminSettingsPayload,
	notices: Vec<DashboardNoticePayload>,
	notices_revision: String,
) -> RuntimeSettingsPayload {
	RuntimeSettingsPayload {
		timezone: payload
			.timezone
			.unwrap_or_else(|| runtime_settings::DEFAULT_TIMEZONE.to_string()),
		meddra_language: payload.meddra_language,
		meddra_version: payload.meddra_version,
		orientation: payload
			.orientation
			.unwrap_or_else(|| "Landscape".to_string()),
		data_ordering: payload
			.data_ordering
			.unwrap_or_else(|| "Primary data will appear first".to_string()),
		notation: payload
			.notation
			.unwrap_or(runtime_settings::DEFAULT_NOTATION),
		apply_sender_info_to_imported_cases: payload
			.apply_sender_info_to_imported_cases
			.unwrap_or(false),
		import_date_update: payload.import_date_update.unwrap_or(
			ImportDateUpdatePayload {
				date_of_creation: Some(false),
				most_recent_info_date: Some(false),
				report_first_received_date: Some(false),
			},
		),
		appendices: normalize_appendices(payload.appendices.as_deref()),
		idle_session_minutes: payload.idle_session_minutes.unwrap_or(60),
		session_warning_minutes: payload.session_warning_minutes.unwrap_or(5),
		notices,
		notices_revision,
	}
}

/// GET /api/settings/runtime
pub async fn get_runtime_settings(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
) -> Result<(StatusCode, Json<RuntimeSettingsPayload>)> {
	let ctx = ctx_w.0;
	let payload = load_admin_settings_payload(&ctx, &mm).await?;
	let notices =
		if notice_read_allowed(&snapshot) || notice_update_allowed(&snapshot) {
			active_notices(
				load_notices(&ctx, &mm).await?,
				payload
					.timezone
					.as_deref()
					.unwrap_or(runtime_settings::DEFAULT_TIMEZONE),
			)?
		} else {
			Vec::new()
		};
	let notices_revision = AdminSettingsBmc::dashboard_notices_revision(&ctx, &mm)
		.await
		.map_err(Error::Model)?;
	Ok((
		StatusCode::OK,
		Json(runtime_settings_payload(payload, notices, notices_revision)),
	))
}

/// GET /api/admin/settings
pub async fn get_admin_settings(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
) -> Result<(StatusCode, Json<AdminSettingsPayload>)> {
	let ctx = ctx_w.0;
	let can_read_notices = notice_read_allowed(&snapshot);
	with_authorized_settings_read(&ctx, &snapshot, &mm, move |ctx, mm| {
		Box::pin(async move {
			let mut payload = load_admin_settings_payload(ctx, mm).await?;
			if !can_read_notices {
				payload.notices = Some(Vec::new());
			}
			Ok((StatusCode::OK, Json(payload)))
		})
	})
	.await
}

/// PUT /api/admin/settings
pub async fn update_admin_settings(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Json(payload): Json<
		lib_rest_core::rest_params::ParamsForUpdate<AdminSettingsUpdateBody>,
	>,
) -> Result<(StatusCode, Json<AdminSettingsPayload>)> {
	let ctx = ctx_w.0;
	with_authorized_settings_update(&ctx, &snapshot, &mm, move |ctx, mm| {
		Box::pin(async move {
			let existing = AdminSettingsBmc::get(ctx, mm, SETTINGS_KEY)
				.await
				.map_err(Error::Model)?;
			let value =
				payload_to_value(ctx, mm, existing.as_ref(), &payload.data).await?;
			let updated_by: Option<Uuid> = Some(ctx.user_id());
			AdminSettingsBmc::upsert(ctx, mm, SETTINGS_KEY, &value, updated_by)
				.await
				.map_err(Error::Model)?;
			let response = serde_json::from_value::<AdminSettingsPayload>(value)?;
			Ok((StatusCode::OK, Json(response)))
		})
	})
	.await
}

/// PUT /api/admin/notices
pub async fn update_admin_notices(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Json(payload): Json<
		lib_rest_core::rest_params::ParamsForUpdate<AdminNoticesUpdateBody>,
	>,
) -> Result<(StatusCode, Json<AdminNoticesPayload>)> {
	let ctx = ctx_w.0;
	with_authorized_notice_update(&ctx, &snapshot, &mm, move |ctx, mm| {
		Box::pin(async move {
			let expected_revision =
				payload.data.revision.as_deref().ok_or_else(|| {
					Error::BadRequest {
					message: "notice revision is required; reload notices before saving"
						.to_string(),
				}
				})?;
			let current_revision =
				AdminSettingsBmc::dashboard_notices_revision(ctx, mm)
					.await
					.map_err(Error::Model)?;
			if expected_revision != current_revision {
				return Err(Error::BadRequest {
					message: "notices changed on the server; reload before saving"
						.to_string(),
				});
			}
			let writer = current_user_email(ctx, mm, ctx.user_id()).await?;
			let notices = normalize_notices(payload.data.notices, writer)?;
			let values = notices
				.iter()
				.map(serde_json::to_value)
				.collect::<std::result::Result<Vec<_>, _>>()
				.map_err(|err| Error::BadRequest {
					message: format!("failed to serialize notices: {err}"),
				})?;
			AdminSettingsBmc::replace_dashboard_notices(
				ctx,
				mm,
				&values,
				ctx.user_id(),
			)
			.await
			.map_err(Error::Model)?;
			let notices = load_notices(ctx, mm).await?;
			let revision = AdminSettingsBmc::dashboard_notices_revision(ctx, mm)
				.await
				.map_err(Error::Model)?;
			Ok((
				StatusCode::OK,
				Json(AdminNoticesPayload { notices, revision }),
			))
		})
	})
	.await
}
