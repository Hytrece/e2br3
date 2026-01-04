// Section C - Safety Report Identification

use crate::model::base::DbBmc;
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- SafetyReportIdentification

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct SafetyReportIdentification {
	pub id: Uuid,
	pub case_id: Uuid,

	// C.1.2 - Date of Creation (MANDATORY)
	pub transmission_date: Date,

	// C.1.3 - Type of Report (MANDATORY)
	pub report_type: String,

	// C.1.4 - Date Report Was First Received from Source (MANDATORY)
	pub date_first_received_from_source: Date,

	// C.1.5 - Date of Most Recent Information (MANDATORY)
	pub date_of_most_recent_information: Date,

	// C.1.7 - Fulfils Expedited Criteria (MANDATORY)
	pub fulfil_expedited_criteria: bool,

	// C.1.8.1 - Worldwide Unique Case Identification
	pub worldwide_unique_id: Option<String>,

	// C.1.11.2 - Nullification Reason
	pub nullification_reason: Option<String>,

	// Receiver Organization
	pub receiver_organization: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct SafetyReportIdentificationForCreate {
	pub case_id: Uuid,
	pub transmission_date: Date,
	pub report_type: String,
	pub date_first_received_from_source: Date,
	pub date_of_most_recent_information: Date,
	pub fulfil_expedited_criteria: bool,
}

#[derive(Fields, Deserialize)]
pub struct SafetyReportIdentificationForUpdate {
	pub transmission_date: Option<Date>,
	pub report_type: Option<String>,
	pub worldwide_unique_id: Option<String>,
	pub nullification_reason: Option<String>,
	pub receiver_organization: Option<String>,
}

// -- SenderInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct SenderInformation {
	pub id: Uuid,
	pub case_id: Uuid,

	// C.3.1 - Sender Type (MANDATORY)
	pub sender_type: String,

	// C.3.2 - Sender's Organisation (MANDATORY)
	pub organization_name: String,
	pub department: Option<String>,
	pub street_address: Option<String>,
	pub city: Option<String>,
	pub state: Option<String>,
	pub postcode: Option<String>,
	pub country_code: Option<String>,

	// C.3.3 - Person Responsible for Sending
	pub person_title: Option<String>,
	pub person_given_name: Option<String>,
	pub person_middle_name: Option<String>,
	pub person_family_name: Option<String>,

	// C.3.4 - Contact Information
	pub telephone: Option<String>,
	pub fax: Option<String>,
	pub email: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct SenderInformationForCreate {
	pub case_id: Uuid,
	pub sender_type: String,
	pub organization_name: String,
}

#[derive(Fields, Deserialize)]
pub struct SenderInformationForUpdate {
	pub sender_type: Option<String>,
	pub organization_name: Option<String>,
	pub department: Option<String>,
	pub street_address: Option<String>,
	pub city: Option<String>,
	pub person_given_name: Option<String>,
	pub person_family_name: Option<String>,
	pub telephone: Option<String>,
	pub email: Option<String>,
}

// -- PrimarySource

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct PrimarySource {
	pub id: Uuid,
	pub case_id: Uuid,
	pub sequence_number: i32,

	// C.2.r.1 - Reporter's Name
	pub reporter_title: Option<String>,
	pub reporter_given_name: Option<String>,
	pub reporter_middle_name: Option<String>,
	pub reporter_family_name: Option<String>,

	// C.2.r.2 - Reporter's Address
	pub organization: Option<String>,
	pub department: Option<String>,
	pub street: Option<String>,
	pub city: Option<String>,
	pub state: Option<String>,
	pub postcode: Option<String>,
	pub telephone: Option<String>,

	// C.2.r.3 - Country Code
	pub country_code: Option<String>,
	pub email: Option<String>,

	// C.2.r.4 - Qualification (MANDATORY within primary source)
	pub qualification: Option<String>,

	// C.2.r.5 - Primary Source for Regulatory Purposes (MANDATORY)
	pub primary_source_regulatory: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct PrimarySourceForCreate {
	pub case_id: Uuid,
	pub sequence_number: i32,
	pub qualification: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct PrimarySourceForUpdate {
	pub reporter_given_name: Option<String>,
	pub reporter_family_name: Option<String>,
	pub organization: Option<String>,
	pub qualification: Option<String>,
	pub primary_source_regulatory: Option<String>,
}

// -- LiteratureReference

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct LiteratureReference {
	pub id: Uuid,
	pub case_id: Uuid,
	pub reference_text: String,
	pub sequence_number: i32,
	pub created_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct LiteratureReferenceForCreate {
	pub case_id: Uuid,
	pub reference_text: String,
	pub sequence_number: i32,
}

// -- StudyInformation

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct StudyInformation {
	pub id: Uuid,
	pub case_id: Uuid,

	pub study_name: Option<String>,
	pub sponsor_study_number: Option<String>,
	pub study_type_reaction: Option<String>,

	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct StudyInformationForCreate {
	pub case_id: Uuid,
	pub study_name: Option<String>,
	pub sponsor_study_number: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct StudyInformationForUpdate {
	pub study_name: Option<String>,
	pub sponsor_study_number: Option<String>,
	pub study_type_reaction: Option<String>,
}

// -- StudyRegistrationNumber

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct StudyRegistrationNumber {
	pub id: Uuid,
	pub study_information_id: Uuid,
	pub registration_number: String,
	pub country_code: Option<String>,
	pub sequence_number: i32,
}

#[derive(Fields, Deserialize)]
pub struct StudyRegistrationNumberForCreate {
	pub study_information_id: Uuid,
	pub registration_number: String,
	pub country_code: Option<String>,
	pub sequence_number: i32,
}

// -- BMCs (Business Model Controllers)

pub struct SafetyReportIdentificationBmc;
impl DbBmc for SafetyReportIdentificationBmc {
	const TABLE: &'static str = "safety_report_identification";
}

pub struct SenderInformationBmc;
impl DbBmc for SenderInformationBmc {
	const TABLE: &'static str = "sender_information";
}

pub struct PrimarySourceBmc;
impl DbBmc for PrimarySourceBmc {
	const TABLE: &'static str = "primary_sources";
}

pub struct LiteratureReferenceBmc;
impl DbBmc for LiteratureReferenceBmc {
	const TABLE: &'static str = "literature_references";
}

pub struct StudyInformationBmc;
impl DbBmc for StudyInformationBmc {
	const TABLE: &'static str = "study_information";
}

pub struct StudyRegistrationNumberBmc;
impl DbBmc for StudyRegistrationNumberBmc {
	const TABLE: &'static str = "study_registration_numbers";
}
