use crate::error::Error;
use crate::types::ParsedE2b;
use crate::Result;
use serde_json::json;

pub fn parse_e2b_xml(xml: &[u8]) -> Result<ParsedE2b> {
	// Import performs validation before parse; keep parser focused on parsing metadata.
	let root = extract_root_element_name(xml).ok_or_else(|| Error::InvalidXml {
		message: "XML root element missing".to_string(),
		line: None,
		column: None,
	})?;
	let json = json!({
		"root": root,
		"size_bytes": xml.len(),
	});

	Ok(ParsedE2b {
		root_element: root,
		json,
	})
}

fn extract_root_element_name(xml: &[u8]) -> Option<String> {
	let xml_str = std::str::from_utf8(xml).ok()?;
	let start = xml_str.find('<')?;
	let rest = &xml_str[start + 1..];
	if rest.starts_with('?') || rest.starts_with('!') {
		return None;
	}
	let end = rest
		.find(|c: char| c.is_whitespace() || c == '>' || c == '/')
		.unwrap_or(rest.len());
	let name = rest[..end].trim();
	if name.is_empty() {
		None
	} else {
		Some(name.to_string())
	}
}
