// Section F - Tests and Procedures

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::base_uuid;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- TestResult

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct TestResult {
	pub id: Uuid,
	pub case_id: Uuid,
	pub sequence_number: i32,

	// F.r.1 - Test Date
	pub test_date: Option<Date>,

	// F.r.2 - Test Name
	pub test_name: String,

	// F.r.2.1 - Test Name (MedDRA coded)
	pub test_meddra_version: Option<String>,
	pub test_meddra_code: Option<String>,

	// F.r.3.1 - Test Result (coded)
	pub test_result_code: Option<String>,

	// F.r.3.2 - Test Result (value/finding)
	pub test_result_value: Option<String>,

	// F.r.3.3 - Test Result Unit
	pub test_result_unit: Option<String>,

	// F.r.3.4 - Result Unstructured Data
	pub result_unstructured: Option<String>,

	// F.r.4-5 - Normal Range
	pub normal_low_value: Option<String>,
	pub normal_high_value: Option<String>,

	// F.r.6 - Comments
	pub comments: Option<String>,

	// F.r.7 - More Information Available
	pub more_info_available: Option<bool>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct TestResultForCreate {
	pub case_id: Uuid,
	pub sequence_number: i32,
	pub test_name: String,
}

#[derive(Fields, Deserialize)]
pub struct TestResultForUpdate {
	pub test_name: Option<String>,
	pub test_date: Option<Date>,
	pub test_result_value: Option<String>,
	pub test_result_unit: Option<String>,
	pub normal_low_value: Option<String>,
	pub normal_high_value: Option<String>,
	pub comments: Option<String>,
}

// -- BMC

pub struct TestResultBmc;
impl DbBmc for TestResultBmc {
	const TABLE: &'static str = "test_results";
}

impl TestResultBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		test_c: TestResultForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, test_c).await
	}

	pub async fn list_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Vec<TestResult>> {
		let sql = format!(
			"SELECT * FROM {} WHERE case_id = $1 ORDER BY sequence_number",
			Self::TABLE
		);
		let tests = sqlx::query_as::<_, TestResult>(&sql)
			.bind(case_id)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(tests)
	}
}
