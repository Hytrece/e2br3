// Section H - Narrative and Other Information

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- NarrativeInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct NarrativeInformation {
	pub id: Uuid,
	pub case_id: Uuid,

	// H.1 - Case Narrative
	pub case_narrative: String,

	// H.2 - Reporter's Comments
	pub reporter_comments: Option<String>,

	// H.4 - Sender's Comments
	pub sender_comments: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct NarrativeInformationForCreate {
	pub case_id: Uuid,
	pub case_narrative: String,
}

#[derive(Fields, Deserialize)]
pub struct NarrativeInformationForUpdate {
	pub case_narrative: Option<String>,
	pub reporter_comments: Option<String>,
	pub sender_comments: Option<String>,
}

// -- SenderDiagnosis

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct SenderDiagnosis {
	pub id: Uuid,
	pub narrative_id: Uuid,
	pub sequence_number: i32,

	// H.3.r.1 - Diagnosis/Syndrome (MedDRA coded)
	pub diagnosis_meddra_version: Option<String>,
	pub diagnosis_meddra_code: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct SenderDiagnosisForCreate {
	pub narrative_id: Uuid,
	pub sequence_number: i32,
	pub diagnosis_meddra_code: Option<String>,
}

// -- CaseSummaryInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct CaseSummaryInformation {
	pub id: Uuid,
	pub narrative_id: Uuid,
	pub sequence_number: i32,

	// H.5.r.1 - Case Summary Type
	pub summary_type: Option<String>,

	// H.5.r.2 - Case Summary Language
	pub language_code: Option<String>,

	// H.5.r.3 - Text
	pub summary_text: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct CaseSummaryInformationForCreate {
	pub narrative_id: Uuid,
	pub sequence_number: i32,
	pub summary_text: Option<String>,
}

// -- BMCs

pub struct NarrativeInformationBmc;
impl DbBmc for NarrativeInformationBmc {
	const TABLE: &'static str = "narrative_information";
}

impl NarrativeInformationBmc {
	pub async fn get_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Option<NarrativeInformation>> {
		let sql = format!("SELECT * FROM {} WHERE case_id = $1", Self::TABLE);
		let narrative = sqlx::query_as::<_, NarrativeInformation>(&sql)
			.bind(case_id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(narrative)
	}
}

pub struct SenderDiagnosisBmc;
impl DbBmc for SenderDiagnosisBmc {
	const TABLE: &'static str = "sender_diagnoses";
}

pub struct CaseSummaryInformationBmc;
impl DbBmc for CaseSummaryInformationBmc {
	const TABLE: &'static str = "case_summary_information";
}
