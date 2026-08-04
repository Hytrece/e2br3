use crate::ctx::Ctx;
use crate::model::admin_settings::AdminSettingsBmc;
use crate::model::store::set_full_context_from_ctx_dbx;
use crate::model::{ModelManager, Result};
use serde_json::Value;

const SETTINGS_KEY: &str = "system";
const SUPPORTED_CASE_NUMBER_SETTING: &str = "AE Row No.";
const SUPPORTED_CASE_NUMBER_SEQUENCE_CONDITION: &str = "Per sender";

#[derive(Debug, Clone, Copy)]
enum CaseNumberSetting {
	AeRowNumber,
}

struct CaseNumberConfig {
	identifier: String,
	padding: usize,
	setting: CaseNumberSetting,
}

pub struct GeneratedCaseNumber {
	pub safety_report_id: String,
	pub worldwide_unique_id: String,
}

fn setting_string(settings: Option<&Value>, key: &str) -> Option<String> {
	settings
		.and_then(|value| value.get(key))
		.and_then(Value::as_str)
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.map(ToOwned::to_owned)
}

fn setting_padding(settings: &Value) -> Result<usize> {
	let padding = settings
		.get("case_number_padding")
		.and_then(Value::as_i64)
		.ok_or_else(|| crate::model::Error::Validation {
			message: "case_number_padding is required".to_string(),
		})?;
	if padding < 1 {
		return Err(crate::model::Error::Validation {
			message: "case_number_padding must be a positive integer".to_string(),
		});
	}
	usize::try_from(padding).map_err(|_| crate::model::Error::Validation {
		message: "case_number_padding must be a positive integer".to_string(),
	})
}

fn identifier_from_settings(settings: &Value) -> Result<String> {
	setting_string(Some(settings), "case_number_identifier").ok_or_else(|| {
		crate::model::Error::Validation {
			message: "case_number_identifier is required".to_string(),
		}
	})
}

fn load_numbering_config(settings: &Value) -> Result<CaseNumberConfig> {
	let setting =
		setting_string(Some(settings), "case_number_setting").ok_or_else(|| {
			crate::model::Error::Validation {
				message: "case_number_setting is required".to_string(),
			}
		})?;
	if setting != SUPPORTED_CASE_NUMBER_SETTING {
		return Err(crate::model::Error::Validation {
			message: format!(
				"case_number_setting is not supported; expected '{SUPPORTED_CASE_NUMBER_SETTING}'"
			),
		});
	}
	let setting = CaseNumberSetting::AeRowNumber;
	let sequence_condition =
		setting_string(Some(settings), "case_number_sequence_condition")
			.ok_or_else(|| crate::model::Error::Validation {
				message: "case_number_sequence_condition is required".to_string(),
			})?;
	if sequence_condition != SUPPORTED_CASE_NUMBER_SEQUENCE_CONDITION {
		return Err(crate::model::Error::Validation {
			message: format!(
				"case_number_sequence_condition is not supported; expected '{SUPPORTED_CASE_NUMBER_SEQUENCE_CONDITION}'"
			),
		});
	}
	let format_fields = settings
		.get("case_number_format_fields")
		.and_then(Value::as_array)
		.ok_or_else(|| crate::model::Error::Validation {
			message: "case_number_format_fields is required".to_string(),
		})?;
	if format_fields.len() != 1
		|| format_fields[0].as_str().map(str::trim)
			!= Some(SUPPORTED_CASE_NUMBER_SETTING)
	{
		return Err(crate::model::Error::Validation {
			message: format!(
				"case number format is not supported; expected only '{SUPPORTED_CASE_NUMBER_SETTING}'"
			),
		});
	}
	let identifier = identifier_from_settings(settings)?;
	let padding = setting_padding(settings)?;
	Ok(CaseNumberConfig {
		identifier,
		padding,
		setting,
	})
}

pub async fn generate_case_number(
	ctx: &Ctx,
	mm: &ModelManager,
) -> Result<GeneratedCaseNumber> {
	let settings = AdminSettingsBmc::get(ctx, mm, SETTINGS_KEY)
		.await?
		.ok_or_else(|| crate::model::Error::Validation {
			message: "admin case number settings are required".to_string(),
		})?;
	let config = load_numbering_config(&settings)?;

	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	if let Err(err) = set_full_context_from_ctx_dbx(dbx, ctx).await {
		dbx.rollback_txn().await?;
		return Err(err);
	}

	let (count,) = match dbx
		.fetch_one(
			sqlx::query_as::<_, (i64,)>(
				"SELECT COUNT(*)
				 FROM safety_report_identification sri
				 JOIN cases c ON c.id = sri.case_id
				 WHERE c.organization_id = $1
				   AND sri.safety_report_id LIKE $2",
			)
			.bind(ctx.organization_id())
			.bind(format!("{}%", config.identifier)),
		)
		.await
	{
		Ok(row) => row,
		Err(err) => {
			dbx.rollback_txn().await?;
			return Err(crate::model::Error::Store(err.to_string()));
		}
	};
	dbx.commit_txn().await?;

	let mut sequence = count + 1;
	loop {
		let safety_report_id = format!(
			"{}{}",
			config.identifier,
			match config.setting {
				CaseNumberSetting::AeRowNumber =>
					format!("{sequence:0width$}", width = config.padding),
			},
		);
		if !case_number_exists(ctx, mm, &safety_report_id).await? {
			return Ok(GeneratedCaseNumber {
				worldwide_unique_id: safety_report_id.clone(),
				safety_report_id,
			});
		}
		sequence += 1;
	}
}

async fn case_number_exists(
	ctx: &Ctx,
	mm: &ModelManager,
	safety_report_id: &str,
) -> Result<bool> {
	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	if let Err(err) = set_full_context_from_ctx_dbx(dbx, ctx).await {
		dbx.rollback_txn().await?;
		return Err(err);
	}
	let (exists,) = match dbx
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(
				"SELECT EXISTS (SELECT 1 FROM safety_report_identification WHERE safety_report_id = $1)",
			)
			.bind(safety_report_id),
		)
		.await
	{
		Ok(row) => row,
		Err(err) => {
			dbx.rollback_txn().await?;
			return Err(crate::model::Error::Store(err.to_string()));
		}
	};
	dbx.commit_txn().await?;
	Ok(exists)
}
