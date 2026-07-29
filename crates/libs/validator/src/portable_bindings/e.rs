use super::{PortableFieldBinding, PortableValueType};

macro_rules! binding {
	($path:literal, $request:literal, $type:ident, [$($code:literal),+ $(,)?]) => {
		PortableFieldBinding {
			section: "AE",
			frontend_path: $path,
			request_path: $request,
			value_type: PortableValueType::$type,
			rule_codes: &[$($code),+],
			null_flavor_path: None,
		}
	};
	($path:literal, $request:literal, $type:ident, [$($code:literal),+ $(,)?], null: $null:literal) => {
		PortableFieldBinding {
			section: "AE",
			frontend_path: $path,
			request_path: $request,
			value_type: PortableValueType::$type,
			rule_codes: &[$($code),+],
			null_flavor_path: Some($null),
		}
	};
}

pub(super) const BINDINGS: &[PortableFieldBinding] = &[
	binding!(
		"reactions[].primarySourceReaction",
		"primarySourceReaction",
		String,
		["ICH.E.i.1.1a.LENGTH.MAX"]
	),
	binding!(
		"reactions[].reactionLanguage",
		"reactionLanguage",
		String,
		["ICH.E.i.1.1b.LENGTH.MAX"]
	),
	binding!(
		"reactions[].primarySourceReactionTranslation",
		"primarySourceReactionTranslation",
		String,
		["ICH.E.i.1.2.LENGTH.MAX"]
	),
	binding!(
		"reactions[].reactionMeddraVersionLLT",
		"reactionMeddraVersionLLT",
		String,
		["ICH.E.i.2.1a.ALLOWED.VALUE", "ICH.E.i.2.1a.LENGTH.MAX"]
	),
	binding!(
		"reactions[].reactionMeddraCodeLLT",
		"reactionMeddraCodeLLT",
		String,
		["ICH.E.i.2.1b.ALLOWED.VALUE", "ICH.E.i.2.1b.LENGTH.MAX"]
	),
	binding!(
		"reactions[].termHighlighted",
		"termHighlighted",
		String,
		["ICH.E.i.3.1.ALLOWED.VALUE", "ICH.E.i.3.1.LENGTH.MAX"]
	),
	binding!(
		"reactions[].seriousness.criteriaResultsInDeath",
		"seriousness.criteriaResultsInDeath",
		Boolean,
		[
			"ICH.E.i.3.2a.ALLOWED.VALUE",
			"ICH.E.i.3.2a.NULLFLAVOR.ALLOWED"
		]
	),
	binding!(
		"reactions[].seriousness.criteriaLifeThreatening",
		"seriousness.criteriaLifeThreatening",
		Boolean,
		[
			"ICH.E.i.3.2b.ALLOWED.VALUE",
			"ICH.E.i.3.2b.NULLFLAVOR.ALLOWED"
		]
	),
	binding!(
		"reactions[].seriousness.criteriaHospitalization",
		"seriousness.criteriaHospitalization",
		Boolean,
		[
			"ICH.E.i.3.2c.ALLOWED.VALUE",
			"ICH.E.i.3.2c.NULLFLAVOR.ALLOWED"
		]
	),
	binding!(
		"reactions[].seriousness.criteriaDisabling",
		"seriousness.criteriaDisabling",
		Boolean,
		[
			"ICH.E.i.3.2d.ALLOWED.VALUE",
			"ICH.E.i.3.2d.NULLFLAVOR.ALLOWED"
		]
	),
	binding!(
		"reactions[].seriousness.criteriaCongenitalAnomaly",
		"seriousness.criteriaCongenitalAnomaly",
		Boolean,
		[
			"ICH.E.i.3.2e.ALLOWED.VALUE",
			"ICH.E.i.3.2e.NULLFLAVOR.ALLOWED"
		]
	),
	binding!(
		"reactions[].seriousness.criteriaOtherMedicallyImportant",
		"seriousness.criteriaOtherMedicallyImportant",
		Boolean,
		[
			"ICH.E.i.3.2f.ALLOWED.VALUE",
			"ICH.E.i.3.2f.NULLFLAVOR.ALLOWED"
		]
	),
	binding!(
		"reactions[].requiredIntervention",
		"requiredIntervention",
		Boolean,
		[
			"FDA.E.i.3.2h.ALLOWED.VALUE",
			"FDA.E.i.3.2h.NULLFLAVOR.ALLOWED"
		],
		null: "reactions[].requiredIntervention"
	),
	binding!(
		"reactions[].reactionStartDate",
		"reactionStartDate",
		String,
		["ICH.E.i.4.ALLOWED.VALUE", "ICH.E.i.4.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"reactions[].reactionEndDate",
		"reactionEndDate",
		String,
		["ICH.E.i.5.ALLOWED.VALUE", "ICH.E.i.5.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"reactions[].reactionDuration.value",
		"reactionDuration.value",
		Number,
		["ICH.E.i.6a.ALLOWED.VALUE", "ICH.E.i.6a.LENGTH.MAX"]
	),
	binding!(
		"reactions[].reactionDuration.unit",
		"reactionDuration.unit",
		String,
		["ICH.E.i.6b.LENGTH.MAX"]
	),
	binding!(
		"reactions[].reactionOutcome",
		"reactionOutcome",
		String,
		["ICH.E.i.7.ALLOWED.VALUE", "ICH.E.i.7.LENGTH.MAX"]
	),
	binding!(
		"reactions[].medicalConfirmation",
		"medicalConfirmation",
		Boolean,
		["ICH.E.i.8.ALLOWED.VALUE"]
	),
	binding!(
		"reactions[].reactionCountry",
		"reactionCountry",
		String,
		["ICH.E.i.9.LENGTH.MAX"]
	),
];
