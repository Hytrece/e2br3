// Section D - Patient Information

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- PatientInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PatientInformation {
	pub id: Uuid,
	pub case_id: Uuid,

	// D.1 - Patient identification
	pub patient_initials: Option<String>,
	pub patient_given_name: Option<String>,
	pub patient_family_name: Option<String>,

	// D.2 - Age
	pub birth_date: Option<Date>,
	pub age_at_time_of_onset: Option<Decimal>,
	pub age_unit: Option<String>,
	pub gestation_period: Option<Decimal>,
	pub gestation_period_unit: Option<String>,
	pub age_group: Option<String>,

	// D.3-5 - Physical
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub sex: Option<String>,

	// D.6 - Last Menstrual Period
	pub last_menstrual_period_date: Option<Date>,

	// D.7.2 - Medical history
	pub medical_history_text: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct PatientInformationForCreate {
	pub case_id: Uuid,
	pub patient_initials: Option<String>,
	pub sex: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct PatientInformationForUpdate {
	pub patient_initials: Option<String>,
	pub patient_given_name: Option<String>,
	pub patient_family_name: Option<String>,
	pub birth_date: Option<Date>,
	pub age_at_time_of_onset: Option<Decimal>,
	pub age_unit: Option<String>,
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub sex: Option<String>,
	pub medical_history_text: Option<String>,
}

// -- MedicalHistoryEpisode

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct MedicalHistoryEpisode {
	pub id: Uuid,
	pub patient_id: Uuid,
	pub sequence_number: i32,

	// D.7.1.r.1a - Disease/Surgical Procedure
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,

	// D.7.1.r.2-4
	pub start_date: Option<Date>,
	pub continuing: Option<bool>,
	pub end_date: Option<Date>,
	pub comments: Option<String>,

	pub created_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct MedicalHistoryEpisodeForCreate {
	pub patient_id: Uuid,
	pub sequence_number: i32,
	pub meddra_code: Option<String>,
}

// -- PastDrugHistory

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PastDrugHistory {
	pub id: Uuid,
	pub patient_id: Uuid,
	pub sequence_number: i32,

	// D.8.r.1 - Drug Name
	pub drug_name: Option<String>,

	// D.8.r.2-3 - Product IDs
	pub mpid: Option<String>,
	pub mpid_version: Option<String>,
	pub phpid: Option<String>,
	pub phpid_version: Option<String>,

	// D.8.r.4-5 - Dates
	pub start_date: Option<Date>,
	pub end_date: Option<Date>,

	// D.8.r.6a - Indication
	pub indication_meddra_version: Option<String>,
	pub indication_meddra_code: Option<String>,

	pub created_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct PastDrugHistoryForCreate {
	pub patient_id: Uuid,
	pub sequence_number: i32,
	pub drug_name: Option<String>,
}

// -- PatientDeathInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PatientDeathInformation {
	pub id: Uuid,
	pub patient_id: Uuid,

	// D.9.1 - Date of Death
	pub date_of_death: Option<Date>,

	// D.9.3 - Autopsy
	pub autopsy_performed: Option<bool>,

	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct PatientDeathInformationForCreate {
	pub patient_id: Uuid,
	pub date_of_death: Option<Date>,
	pub autopsy_performed: Option<bool>,
}

// -- ReportedCauseOfDeath

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct ReportedCauseOfDeath {
	pub id: Uuid,
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct ReportedCauseOfDeathForCreate {
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_code: Option<String>,
}

// -- AutopsyCauseOfDeath

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct AutopsyCauseOfDeath {
	pub id: Uuid,
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_version: Option<String>,
	pub meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct AutopsyCauseOfDeathForCreate {
	pub death_info_id: Uuid,
	pub sequence_number: i32,
	pub meddra_code: Option<String>,
}

// -- ParentInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct ParentInformation {
	pub id: Uuid,
	pub patient_id: Uuid,

	pub parent_identification: Option<String>,
	pub parent_age: Option<Decimal>,
	pub parent_age_unit: Option<String>,
	pub last_menstrual_period_date: Option<Date>,
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub sex: Option<String>,

	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct ParentInformationForCreate {
	pub patient_id: Uuid,
	pub sex: Option<String>,
}

// -- BMCs

pub struct PatientInformationBmc;
impl DbBmc for PatientInformationBmc {
	const TABLE: &'static str = "patient_information";
}

impl PatientInformationBmc {
	pub async fn get_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Option<PatientInformation>> {
		let sql = format!("SELECT * FROM {} WHERE case_id = $1", Self::TABLE);
		let patient = sqlx::query_as::<_, PatientInformation>(&sql)
			.bind(case_id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(patient)
	}
}

pub struct MedicalHistoryEpisodeBmc;
impl DbBmc for MedicalHistoryEpisodeBmc {
	const TABLE: &'static str = "medical_history_episodes";
}

pub struct PastDrugHistoryBmc;
impl DbBmc for PastDrugHistoryBmc {
	const TABLE: &'static str = "past_drug_history";
}

pub struct PatientDeathInformationBmc;
impl DbBmc for PatientDeathInformationBmc {
	const TABLE: &'static str = "patient_death_information";
}

pub struct ReportedCauseOfDeathBmc;
impl DbBmc for ReportedCauseOfDeathBmc {
	const TABLE: &'static str = "reported_causes_of_death";
}

pub struct AutopsyCauseOfDeathBmc;
impl DbBmc for AutopsyCauseOfDeathBmc {
	const TABLE: &'static str = "autopsy_causes_of_death";
}

pub struct ParentInformationBmc;
impl DbBmc for ParentInformationBmc {
	const TABLE: &'static str = "parent_information";
}
