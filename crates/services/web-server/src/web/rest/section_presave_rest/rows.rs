use serde_json::{Map, Value};

pub(super) fn camelize_rows<T: serde::Serialize>(rows: Vec<T>) -> Vec<Value> {
	rows.into_iter()
		.map(|row| {
			camelize_value(
				serde_json::to_value(row).expect("serializable presave row"),
			)
		})
		.collect()
}

pub(super) fn camelize_value(value: Value) -> Value {
	match value {
		Value::Object(values) => Value::Object(
			values
				.into_iter()
				.map(|(key, value)| (camelize_key(&key), camelize_value(value)))
				.collect::<Map<_, _>>(),
		),
		Value::Array(values) => {
			Value::Array(values.into_iter().map(camelize_value).collect())
		}
		value => value,
	}
}

fn camelize_key(value: &str) -> String {
	let mut output = String::with_capacity(value.len());
	let mut uppercase = false;
	for character in value.chars() {
		if character == '_' {
			uppercase = true;
		} else if uppercase {
			output.extend(character.to_uppercase());
			uppercase = false;
		} else {
			output.push(character);
		}
	}
	output
}
