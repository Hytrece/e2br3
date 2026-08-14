use serde::Deserialize;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PatchValue<T> {
	#[default]
	Missing,
	Null,
	Value(T),
}

impl<T> PatchValue<T> {
	pub fn into_parts(self) -> (Option<T>, bool) {
		match self {
			Self::Missing => (None, false),
			Self::Null => (None, true),
			Self::Value(value) => (Some(value), false),
		}
	}
}

pub fn deserialize_patch_value<'de, D, T>(
	deserializer: D,
) -> std::result::Result<PatchValue<T>, D::Error>
where
	D: serde::Deserializer<'de>,
	T: Deserialize<'de>,
{
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum PatchInput<T> {
		Null(()),
		Value(T),
	}

	Ok(match PatchInput::<T>::deserialize(deserializer)? {
		PatchInput::Null(()) => PatchValue::Null,
		PatchInput::Value(value) => PatchValue::Value(value),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Deserialize)]
	struct PatchRequest {
		#[serde(default, deserialize_with = "deserialize_patch_value")]
		value: PatchValue<String>,
	}

	#[test]
	fn distinguishes_missing_null_and_value() {
		let missing: PatchRequest = serde_json::from_str("{}").unwrap();
		let null: PatchRequest = serde_json::from_str(r#"{"value":null}"#).unwrap();
		let value: PatchRequest =
			serde_json::from_str(r#"{"value":"present"}"#).unwrap();

		assert_eq!(missing.value.into_parts(), (None, false));
		assert_eq!(null.value.into_parts(), (None, true));
		assert_eq!(
			value.value.into_parts(),
			(Some("present".to_string()), false)
		);
	}
}
