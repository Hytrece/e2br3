// Section N - Batch/Message Headers

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

// -- MessageHeader

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct MessageHeader {
	pub id: Uuid,
	pub case_id: Uuid,

	// N.1.1 - Batch Number
	pub batch_number: Option<String>,

	// N.1.2 - Batch Sender Identifier
	pub batch_sender_identifier: Option<String>,

	// Message identification
	pub message_type: String,           // ichicsr
	pub message_format_version: String, // 2.1
	pub message_format_release: String, // 2.0
	pub message_number: String,
	pub message_sender_identifier: String,
	pub message_receiver_identifier: String,
	pub message_date_format: String, // 204 (CCYYMMDDHHMMSS)
	pub message_date: String,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct MessageHeaderForCreate {
	pub case_id: Uuid,
	pub message_number: String,
	pub message_sender_identifier: String,
	pub message_receiver_identifier: String,
	pub message_date: String,
}

#[derive(Fields, Deserialize)]
pub struct MessageHeaderForUpdate {
	pub batch_number: Option<String>,
	pub message_number: Option<String>,
	pub message_sender_identifier: Option<String>,
	pub message_receiver_identifier: Option<String>,
}

// -- BMC

pub struct MessageHeaderBmc;
impl DbBmc for MessageHeaderBmc {
	const TABLE: &'static str = "message_headers";
}

impl MessageHeaderBmc {
	pub async fn get_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Option<MessageHeader>> {
		let sql = format!("SELECT * FROM {} WHERE case_id = $1", Self::TABLE);
		let header = sqlx::query_as::<_, MessageHeader>(&sql)
			.bind(case_id)
			.fetch_optional(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(header)
	}
}
