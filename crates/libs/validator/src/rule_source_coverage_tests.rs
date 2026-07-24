use crate::{
	canonical_rules_for_phase, portable_constraints, portable_field_bindings,
	ValidationPhase,
};
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
fn executable_rule_source_references_exist_in_compiled_catalogs() {
	let coverage: Coverage = serde_json::from_str(include_str!(
		"../../../../registry/rule-source-coverage.json"
	))
	.expect("rule source coverage JSON parses");
	let business = canonical_rules_for_phase(ValidationPhase::CaseValidate)
		.into_iter()
		.map(|rule| rule.code.to_string())
		.collect::<BTreeSet<_>>();
	let portable = portable_constraints()
		.into_iter()
		.map(|rule| rule.code)
		.collect::<BTreeSet<_>>();
	let bound = portable_field_bindings()
		.into_iter()
		.flat_map(|binding| binding.rule_codes.iter().copied())
		.collect::<BTreeSet<_>>();

	for source in coverage.sources {
		for requirement in source.requirements {
			match requirement.disposition.as_str() {
				"business_rule" => {
					for code in requirement.rule_codes {
						assert!(
							business.contains(&code),
							"{} references missing business rule {}",
							requirement.id,
							code
						);
					}
				}
				"constraint" => {
					for code in requirement.rule_codes {
						assert!(
							portable.contains(&code),
							"{} references missing portable constraint {}",
							requirement.id,
							code
						);
						assert!(
							bound.contains(code.as_str()),
							"{} references unbound portable constraint {}",
							requirement.id,
							code
						);
					}
				}
				"guidance" | "deferred" => {}
				other => panic!("unsupported source disposition {other}"),
			}
		}
	}
}
