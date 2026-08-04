use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RegulatoryAuthority {
	Ich,
	Fda,
	Mfds,
}

impl RegulatoryAuthority {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Ich => "ich",
			Self::Fda => "fda",
			Self::Mfds => "mfds",
		}
	}

	pub fn parse(value: &str) -> Option<Self> {
		match value.trim().to_ascii_lowercase().as_str() {
			"ich" => Some(Self::Ich),
			"fda" => Some(Self::Fda),
			"mfds" => Some(Self::Mfds),
			_ => None,
		}
	}

	pub fn from_case_authority(value: Option<&str>) -> Option<Self> {
		value.and_then(Self::parse)
	}

	pub fn requires_fda_context(self) -> bool {
		matches!(self, Self::Fda)
	}

	pub fn requires_mfds_context(self) -> bool {
		matches!(self, Self::Mfds)
	}
}

pub fn fda_attachment_media_type(file_name: &str) -> Option<&'static str> {
	match file_name.rsplit_once('.')?.1.to_ascii_lowercase().as_str() {
		"pdf" => Some("application/pdf"),
		"jpeg" | "jpg" => Some("image/jpeg"),
		"bmp" => Some("image/bmp"),
		"png" => Some("image/png"),
		"gif" => Some("image/gif"),
		"tiff" => Some("image/tiff"),
		"tif" => Some("image/tif"),
		"txt" => Some("text/plain"),
		"xls" => Some("application/vnd.ms-excel"),
		"xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
		"doc" => Some("application/msword"),
		"docx" => Some(
			"application/vnd.openxmlformats-officedocument.wordprocessingml.document",
		),
		"wpd" => Some("application/vnd.wordperfect"),
		_ => None,
	}
}

pub const FDA_BATCH_RECEIVER_POSTMARKET: &str = "ZZFDA";
pub const FDA_BATCH_RECEIVER_PREMARKET: &str = "ZZFDA_PREMKT";
pub const FDA_MSG_RECEIVER_CDER: &str = "CDER";
pub const FDA_MSG_RECEIVER_CBER: &str = "CBER";
pub const FDA_MSG_RECEIVER_CDER_IND: &str = "CDER_IND";
pub const FDA_MSG_RECEIVER_CBER_IND: &str = "CBER_IND";
pub const FDA_MSG_RECEIVER_CDER_IND_EXEMPT_BA_BE: &str = "CDER_IND_EXEMPT_BA_BE";
/// MFDS receiver identifiers used identically in N.1.4 and N.2.r.3.
pub const MFDS_BATCH_RECEIVER_POSTMARKET_DOMESTIC: &str = "MFDS-O-KR";
pub const MFDS_BATCH_RECEIVER_POSTMARKET_FOREIGN: &str = "MFDS-O-FR";
pub const MFDS_BATCH_RECEIVER_CLINICAL_TRIAL: &str = "MFDS-O-CT";
pub const MFDS_BATCH_RECEIVER_FOREIGN_CLINICAL_TRIAL: &str = "MFDS-O-CF";
pub const MFDS_BATCH_RECEIVER_COMPASSIONATE_USE: &str = "MFDS-O-CU";
pub const MFDS_TEST_RECEIVER_POSTMARKET_DOMESTIC: &str = "MFDS-T-KR";
pub const MFDS_TEST_RECEIVER_POSTMARKET_FOREIGN: &str = "MFDS-T-FR";
pub const MFDS_TEST_RECEIVER_CLINICAL_TRIAL: &str = "MFDS-T-CT";
pub const MFDS_TEST_RECEIVER_FOREIGN_CLINICAL_TRIAL: &str = "MFDS-T-CF";
pub const MFDS_TEST_RECEIVER_COMPASSIONATE_USE: &str = "MFDS-T-CU";

/// Known valid MFDS N.1.4/N.2.r.3 receiver codes.
pub const MFDS_KNOWN_RECEIVERS: &[&str] = &[
	MFDS_BATCH_RECEIVER_POSTMARKET_DOMESTIC,
	MFDS_BATCH_RECEIVER_POSTMARKET_FOREIGN,
	MFDS_BATCH_RECEIVER_CLINICAL_TRIAL,
	MFDS_BATCH_RECEIVER_FOREIGN_CLINICAL_TRIAL,
	MFDS_BATCH_RECEIVER_COMPASSIONATE_USE,
	MFDS_TEST_RECEIVER_POSTMARKET_DOMESTIC,
	MFDS_TEST_RECEIVER_POSTMARKET_FOREIGN,
	MFDS_TEST_RECEIVER_CLINICAL_TRIAL,
	MFDS_TEST_RECEIVER_FOREIGN_CLINICAL_TRIAL,
	MFDS_TEST_RECEIVER_COMPASSIONATE_USE,
];

fn is_one_of(value: Option<&str>, expected: &[&str]) -> bool {
	value
		.map(str::trim)
		.is_some_and(|value| expected.contains(&value))
}

/// Returns true if the official MFDS receiver identifies domestic (KR) reporting.
pub fn is_mfds_domestic_receiver(value: Option<&str>) -> bool {
	is_one_of(
		value,
		&[
			MFDS_BATCH_RECEIVER_POSTMARKET_DOMESTIC,
			MFDS_TEST_RECEIVER_POSTMARKET_DOMESTIC,
		],
	)
}

/// Returns true if the official MFDS receiver identifies foreign postmarket (FR) reporting.
pub fn is_mfds_foreign_postmarket_receiver(value: Option<&str>) -> bool {
	is_one_of(
		value,
		&[
			MFDS_BATCH_RECEIVER_POSTMARKET_FOREIGN,
			MFDS_TEST_RECEIVER_POSTMARKET_FOREIGN,
		],
	)
}

/// Returns true for domestic or foreign MFDS clinical-trial reporting.
pub fn is_mfds_clinical_trial_receiver(value: Option<&str>) -> bool {
	is_one_of(
		value,
		&[
			MFDS_BATCH_RECEIVER_CLINICAL_TRIAL,
			MFDS_BATCH_RECEIVER_FOREIGN_CLINICAL_TRIAL,
			MFDS_TEST_RECEIVER_CLINICAL_TRIAL,
			MFDS_TEST_RECEIVER_FOREIGN_CLINICAL_TRIAL,
		],
	)
}

/// Returns true if the official MFDS receiver identifies compassionate use (CU) reporting.
pub fn is_mfds_compassionate_use_receiver(value: Option<&str>) -> bool {
	is_one_of(
		value,
		&[
			MFDS_BATCH_RECEIVER_COMPASSIONATE_USE,
			MFDS_TEST_RECEIVER_COMPASSIONATE_USE,
		],
	)
}

pub fn is_fda_batch_receiver(value: Option<&str>) -> bool {
	matches!(
		value,
		Some(FDA_BATCH_RECEIVER_POSTMARKET | FDA_BATCH_RECEIVER_PREMARKET)
	)
}

pub fn is_fda_postmarket_batch_receiver(value: Option<&str>) -> bool {
	value == Some(FDA_BATCH_RECEIVER_POSTMARKET)
}

pub fn is_fda_premarket_batch_receiver(value: Option<&str>) -> bool {
	value == Some(FDA_BATCH_RECEIVER_PREMARKET)
}

pub fn is_fda_message_receiver(value: Option<&str>) -> bool {
	matches!(
		value,
		Some(
			FDA_MSG_RECEIVER_CDER
				| FDA_MSG_RECEIVER_CBER
				| FDA_MSG_RECEIVER_CDER_IND
				| FDA_MSG_RECEIVER_CBER_IND
				| FDA_MSG_RECEIVER_CDER_IND_EXEMPT_BA_BE
		)
	)
}

pub fn is_fda_postmarket_message_receiver(value: Option<&str>) -> bool {
	matches!(value, Some(FDA_MSG_RECEIVER_CDER | FDA_MSG_RECEIVER_CBER))
}

pub fn is_fda_ind_message_receiver(value: Option<&str>) -> bool {
	matches!(
		value,
		Some(FDA_MSG_RECEIVER_CDER_IND | FDA_MSG_RECEIVER_CBER_IND)
	)
}

pub fn is_fda_premarket_message_receiver(value: Option<&str>) -> bool {
	matches!(
		value,
		Some(
			FDA_MSG_RECEIVER_CDER_IND
				| FDA_MSG_RECEIVER_CBER_IND
				| FDA_MSG_RECEIVER_CDER_IND_EXEMPT_BA_BE
		)
	)
}

pub fn is_fda_pre_anda_message_receiver(value: Option<&str>) -> bool {
	value == Some(FDA_MSG_RECEIVER_CDER_IND_EXEMPT_BA_BE)
}

pub fn is_mfds_receiver(value: Option<&str>) -> bool {
	value
		.map(str::trim)
		.is_some_and(|value| MFDS_KNOWN_RECEIVERS.contains(&value))
}

pub fn infer_regulatory_authority_from_receivers(
	batch_receiver: Option<&str>,
	message_receiver: Option<&str>,
) -> RegulatoryAuthority {
	if is_mfds_receiver(batch_receiver) || is_mfds_receiver(message_receiver) {
		return RegulatoryAuthority::Mfds;
	}
	if is_fda_batch_receiver(batch_receiver)
		|| is_fda_message_receiver(message_receiver)
	{
		return RegulatoryAuthority::Fda;
	}
	RegulatoryAuthority::Ich
}
