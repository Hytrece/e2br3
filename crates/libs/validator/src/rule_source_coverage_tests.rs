use crate::case_validation_rules;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Coverage {
	sources: Vec<SourceCoverage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceCoverage {
	requirements: Vec<Requirement>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Requirement {
	id: String,
	disposition: String,
	#[serde(default)]
	rule_codes: Vec<String>,
}

#[test]
fn executable_rule_source_references_exist_in_compiled_rules() {
	let coverage: Coverage = serde_json::from_str(include_str!(
		"../../../../registry/rule-source-coverage.json"
	))
	.expect("rule source coverage JSON parses");
	let business = case_validation_rules()
		.into_iter()
		.map(|rule| rule.code.to_string())
		.collect::<BTreeSet<_>>();
	let input_contracts = [
		include_str!("../../input-contracts/src/generated/c.rs"),
		include_str!("../../input-contracts/src/generated/d.rs"),
		include_str!("../../input-contracts/src/generated/e.rs"),
		include_str!("../../input-contracts/src/generated/f.rs"),
		include_str!("../../input-contracts/src/generated/g.rs"),
		include_str!("../../input-contracts/src/generated/h.rs"),
		include_str!("../../input-contracts/src/generated/n.rs"),
	]
	.into_iter()
	.flat_map(str::lines)
	.filter_map(|line| line.strip_prefix("/// "))
	.collect::<BTreeSet<_>>();
	let mut missing = Vec::new();

	for source in coverage.sources {
		for requirement in source.requirements {
			match requirement.disposition.as_str() {
				"business_rule" => {
					for code in requirement.rule_codes {
						if !business.contains(&code) {
							missing.push(format!(
								"{} references missing business rule {}",
								requirement.id, code
							));
						}
					}
				}
				"constraint" => {
					for code in requirement.rule_codes {
						if !input_contracts.contains(code.as_str()) {
							missing.push(format!(
								"{} references missing input contract {}",
								requirement.id, code
							));
						}
					}
				}
				"guidance" | "deferred" => {}
				other => panic!("unsupported source disposition {other}"),
			}
		}
	}
	assert!(missing.is_empty(), "{}", missing.join("\n"));
}
