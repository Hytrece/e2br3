use super::{PortableBindingExclusion, PortableFieldBinding, PortableValueType};

macro_rules! binding {
	($path:literal, $request:literal, $type:ident, [$($code:literal),+ $(,)?]) => {
		PortableFieldBinding {
			section: "DG",
			frontend_path: $path,
			request_path: $request,
			value_type: PortableValueType::$type,
			rule_codes: &[$($code),+],
			null_flavor_path: None,
		}
	};
	($path:literal, $request:literal, $type:ident, [$($code:literal),* $(,)?], null: $null:literal) => {
		PortableFieldBinding {
			section: "DG",
			frontend_path: $path,
			request_path: $request,
			value_type: PortableValueType::$type,
			rule_codes: &[$($code),*],
			null_flavor_path: Some($null),
		}
	};
}

pub(super) const BINDINGS: &[PortableFieldBinding] = &[
	binding!(
		"drugs[].drugCharacterization",
		"drugCharacterization",
		String,
		["ICH.G.k.1.ALLOWED.VALUE", "ICH.G.k.1.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaOtherCharacterization",
		"fdaOtherCharacterization",
		String,
		["FDA.G.k.1.a.LENGTH.MAX", "FDA.G.k.1.a.ALLOWED.VALUE"]
	),
	binding!(
		"drugs[].mpidVersion",
		"mpidVersion",
		String,
		["ICH.G.k.2.1.1a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].mpid",
		"mpid",
		String,
		["ICH.G.k.2.1.1b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].phpidVersion",
		"phpidVersion",
		String,
		["ICH.G.k.2.1.2a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].phpid",
		"phpid",
		String,
		["ICH.G.k.2.1.2b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].mfdsMpidVersion",
		"mfdsMpidVersion",
		String,
		["MFDS.G.k.2.1.KR.1a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].mfdsMpid",
		"mfdsMpid",
		String,
		["MFDS.G.k.2.1.KR.1b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].medicinalProduct",
		"medicinalProduct",
		String,
		["ICH.G.k.2.2.LENGTH.MAX"]
	),
	binding!(
		"drugs[].obtainDrugCountry",
		"obtainDrugCountry",
		String,
		["ICH.G.k.2.4.LENGTH.MAX"]
	),
	binding!(
		"drugs[].investigationalProductBlinded",
		"investigationalProductBlinded",
		Boolean,
		["ICH.G.k.2.5.ALLOWED.VALUE"]
	),
	binding!(
		"drugs[].drugAuthorizationNumber",
		"drugAuthorizationNumber",
		String,
		["ICH.G.k.3.1.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugAuthorizationCountry",
		"drugAuthorizationCountry",
		String,
		["ICH.G.k.3.2.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugAuthorizationHolder",
		"drugAuthorizationHolder",
		String,
		["ICH.G.k.3.3.LENGTH.MAX"]
	),
	binding!(
		"drugs[].cumulativeDoseValue",
		"cumulativeDoseValue",
		Number,
		["ICH.G.k.5a.ALLOWED.VALUE", "ICH.G.k.5a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].cumulativeDoseUnit",
		"cumulativeDoseUnit",
		String,
		["ICH.G.k.5b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].gestationPeriodExposureValue",
		"gestationPeriodExposureValue",
		Number,
		["ICH.G.k.6a.ALLOWED.VALUE", "ICH.G.k.6a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].gestationPeriodExposureUnit",
		"gestationPeriodExposureUnit",
		String,
		["ICH.G.k.6b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugActionTaken",
		"drugActionTaken",
		String,
		["ICH.G.k.8.ALLOWED.VALUE", "ICH.G.k.8.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugAdditionalInformationCodes[]",
		"drugAdditionalInformationCodes[]",
		String,
		["ICH.G.k.10.r.ALLOWED.VALUE", "ICH.G.k.10.r.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaAdditionalInfoCoded",
		"fdaAdditionalInfoCoded",
		String,
		["FDA.G.k.10a.LENGTH.MAX", "FDA.G.k.10a.ALLOWED.VALUE"]
	),
	binding!(
		"drugs[].fdaSpecializedProductCategory",
		"fdaSpecializedProductCategory",
		String,
		["FDA.G.k.10.1.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugAdditionalInformation",
		"drugAdditionalInformation",
		String,
		["ICH.G.k.11.LENGTH.MAX"]
	),
	binding!(
		"drugs[].activeSubstances[].substanceName",
		"activeSubstances[].substanceName",
		String,
		["ICH.G.k.2.3.r.1.LENGTH.MAX"]
	),
	binding!(
		"drugs[].activeSubstances[].mfdsVersion",
		"activeSubstances[].mfdsVersion",
		String,
		["MFDS.G.k.2.3.r.1.KR.1a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].activeSubstances[].mfdsId",
		"activeSubstances[].mfdsId",
		String,
		["MFDS.G.k.2.3.r.1.KR.1b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].activeSubstances[].substanceTermIdVersion",
		"activeSubstances[].substanceTermIdVersion",
		String,
		["ICH.G.k.2.3.r.2a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].activeSubstances[].substanceTermId",
		"activeSubstances[].substanceTermId",
		String,
		["ICH.G.k.2.3.r.2b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].activeSubstances[].substanceStrengthValue",
		"activeSubstances[].substanceStrengthValue",
		Number,
		[
			"ICH.G.k.2.3.r.3a.ALLOWED.VALUE",
			"ICH.G.k.2.3.r.3a.LENGTH.MAX"
		]
	),
	binding!(
		"drugs[].activeSubstances[].substanceStrengthUnit",
		"activeSubstances[].substanceStrengthUnit",
		String,
		["ICH.G.k.2.3.r.3b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].doseValue",
		"dosageInformation[].doseValue",
		Number,
		["ICH.G.k.4.r.1a.ALLOWED.VALUE", "ICH.G.k.4.r.1a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].doseUnit",
		"dosageInformation[].doseUnit",
		String,
		["ICH.G.k.4.r.1b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].numberOfUnits",
		"dosageInformation[].numberOfUnits",
		Number,
		["ICH.G.k.4.r.2.ALLOWED.VALUE", "ICH.G.k.4.r.2.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].frequencyUnit",
		"dosageInformation[].frequencyUnit",
		String,
		["ICH.G.k.4.r.3.LENGTH.MAX"]
	),
	binding!("drugs[].dosageInformation[].firstAdministrationDate", "dosageInformation[].firstAdministrationDate", String, [], null: "drugs[].dosageInformation[].firstAdministrationDateNullFlavor"),
	binding!(
		"drugs[].dosageInformation[].firstAdministrationDateNullFlavor",
		"dosageInformation[].firstAdministrationDateNullFlavor",
		String,
		["ICH.G.k.4.r.4.NULLFLAVOR.ALLOWED"]
	),
	binding!("drugs[].dosageInformation[].lastAdministrationDate", "dosageInformation[].lastAdministrationDate", String, [], null: "drugs[].dosageInformation[].lastAdministrationDateNullFlavor"),
	binding!(
		"drugs[].dosageInformation[].lastAdministrationDateNullFlavor",
		"dosageInformation[].lastAdministrationDateNullFlavor",
		String,
		["ICH.G.k.4.r.5.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"drugs[].dosageInformation[].durationValue",
		"dosageInformation[].durationValue",
		Number,
		["ICH.G.k.4.r.6a.ALLOWED.VALUE", "ICH.G.k.4.r.6a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].durationUnit",
		"dosageInformation[].durationUnit",
		String,
		["ICH.G.k.4.r.6b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].batchNumber",
		"dosageInformation[].batchNumber",
		String,
		["ICH.G.k.4.r.7.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].dosageText",
		"dosageInformation[].dosageText",
		String,
		["ICH.G.k.4.r.8.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].doseForm",
		"dosageInformation[].doseForm",
		String,
		["ICH.G.k.4.r.9.1.LENGTH.MAX"],
		null: "drugs[].dosageInformation[].doseFormNullFlavor"
	),
	binding!(
		"drugs[].dosageInformation[].doseFormNullFlavor",
		"dosageInformation[].doseFormNullFlavor",
		String,
		["ICH.G.k.4.r.9.1.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"drugs[].dosageInformation[].doseFormTermIdVersion",
		"dosageInformation[].doseFormTermIdVersion",
		String,
		["ICH.G.k.4.r.9.2a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].doseFormTermId",
		"dosageInformation[].doseFormTermId",
		String,
		["ICH.G.k.4.r.9.2b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].routeOfAdministration",
		"dosageInformation[].routeOfAdministration",
		String,
		["ICH.G.k.4.r.10.1.LENGTH.MAX"],
		null: "drugs[].dosageInformation[].routeOfAdministrationNullFlavor"
	),
	binding!(
		"drugs[].dosageInformation[].routeOfAdministrationNullFlavor",
		"dosageInformation[].routeOfAdministrationNullFlavor",
		String,
		["ICH.G.k.4.r.10.1.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"drugs[].dosageInformation[].routeTermIdVersion",
		"dosageInformation[].routeTermIdVersion",
		String,
		["ICH.G.k.4.r.10.2a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].routeTermId",
		"dosageInformation[].routeTermId",
		String,
		["ICH.G.k.4.r.10.2b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].parentRouteOfAdministration",
		"dosageInformation[].parentRouteOfAdministration",
		String,
		["ICH.G.k.4.r.11.1.LENGTH.MAX"],
		null: "drugs[].dosageInformation[].parentRouteOfAdministrationNullFlavor"
	),
	binding!(
		"drugs[].dosageInformation[].parentRouteOfAdministrationNullFlavor",
		"dosageInformation[].parentRouteOfAdministrationNullFlavor",
		String,
		["ICH.G.k.4.r.11.1.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"drugs[].dosageInformation[].parentRouteTermIdVersion",
		"dosageInformation[].parentRouteTermIdVersion",
		String,
		["ICH.G.k.4.r.11.2a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].dosageInformation[].parentRouteTermId",
		"dosageInformation[].parentRouteTermId",
		String,
		["ICH.G.k.4.r.11.2b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].indications[].indicationText",
		"indications[].indicationText",
		String,
		["ICH.G.k.7.r.1.LENGTH.MAX"],
		null: "drugs[].indications[].indicationTextNullFlavor"
	),
	binding!(
		"drugs[].indications[].indicationTextNullFlavor",
		"indications[].indicationTextNullFlavor",
		String,
		["ICH.G.k.7.r.1.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"drugs[].indications[].indicationMeddraVersion",
		"indications[].indicationMeddraVersion",
		String,
		["ICH.G.k.7.r.2a.ALLOWED.VALUE", "ICH.G.k.7.r.2a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].indications[].indicationMeddraCode",
		"indications[].indicationMeddraCode",
		String,
		["ICH.G.k.7.r.2b.ALLOWED.VALUE", "ICH.G.k.7.r.2b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugReactionAssessments[].administrationStartIntervalValue",
		"drugReactionAssessments[].administrationStartIntervalValue",
		Number,
		[
			"ICH.G.k.9.i.3.1a.ALLOWED.VALUE",
			"ICH.G.k.9.i.3.1a.LENGTH.MAX"
		]
	),
	binding!(
		"drugs[].drugReactionAssessments[].administrationStartIntervalUnit",
		"drugReactionAssessments[].administrationStartIntervalUnit",
		String,
		["ICH.G.k.9.i.3.1b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugReactionAssessments[].lastDoseIntervalValue",
		"drugReactionAssessments[].lastDoseIntervalValue",
		Number,
		[
			"ICH.G.k.9.i.3.2a.ALLOWED.VALUE",
			"ICH.G.k.9.i.3.2a.LENGTH.MAX"
		]
	),
	binding!(
		"drugs[].drugReactionAssessments[].lastDoseIntervalUnit",
		"drugReactionAssessments[].lastDoseIntervalUnit",
		String,
		["ICH.G.k.9.i.3.2b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugReactionAssessments[].sourceOfAssessment",
		"drugReactionAssessments[].sourceOfAssessment",
		String,
		["ICH.G.k.9.i.2.r.1.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugReactionAssessments[].methodOfAssessment",
		"drugReactionAssessments[].methodOfAssessment",
		String,
		["ICH.G.k.9.i.2.r.2.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugReactionAssessments[].methodOfAssessmentKr1",
		"drugReactionAssessments[].methodOfAssessmentKr1",
		String,
		[
			"MFDS.G.k.9.i.2.r.2.KR.1.ALLOWED.VALUE",
			"MFDS.G.k.9.i.2.r.2.KR.1.LENGTH.MAX"
		]
	),
	binding!(
		"drugs[].drugReactionAssessments[].resultOfAssessment",
		"drugReactionAssessments[].resultOfAssessment",
		String,
		["ICH.G.k.9.i.2.r.3.LENGTH.MAX"]
	),
	binding!(
		"drugs[].drugReactionAssessments[].resultOfAssessmentKr1",
		"drugReactionAssessments[].resultOfAssessmentKr1",
		String,
		[
			"MFDS.G.k.9.i.2.r.3.KR.1.ALLOWED.VALUE",
			"MFDS.G.k.9.i.2.r.3.KR.1.LENGTH.MAX"
		],
		null: "drugs[].drugReactionAssessments[].resultOfAssessmentKr1NullFlavor"
	),
	binding!(
		"drugs[].drugReactionAssessments[].resultOfAssessmentKr1NullFlavor",
		"drugReactionAssessments[].resultOfAssessmentKr1NullFlavor",
		String,
		["MFDS.G.k.9.i.2.r.3.KR.1.NULLFLAVOR.ALLOWED"]
	),
	binding!(
		"drugs[].drugReactionAssessments[].resultOfAssessmentKr2",
		"drugReactionAssessments[].resultOfAssessmentKr2",
		String,
		[
			"MFDS.G.k.9.i.2.r.3.KR.2.ALLOWED.VALUE",
			"MFDS.G.k.9.i.2.r.3.KR.2.LENGTH.MAX"
		]
	),
	binding!(
		"drugs[].drugReactionAssessments[].recurrenceAction",
		"drugReactionAssessments[].recurrenceAction",
		String,
		["ICH.G.k.9.i.4.ALLOWED.VALUE", "ICH.G.k.9.i.4.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].followUpTypes[].valueCode",
		"fdaDevices[].followUpTypes[].valueCode",
		String,
		[
			"FDA.G.k.12.r.2.r.LENGTH.MAX",
			"FDA.G.k.12.r.2.r.ALLOWED.VALUE"
		]
	),
	binding!(
		"drugs[].fdaDevices[].deviceProblemCodes[].valueCode",
		"fdaDevices[].deviceProblemCodes[].valueCode",
		String,
		["FDA.G.k.12.r.3.r.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].deviceBrandName",
		"fdaDevices[].deviceBrandName",
		String,
		["FDA.G.k.12.r.4.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].commonDeviceName",
		"fdaDevices[].commonDeviceName",
		String,
		["FDA.G.k.12.r.5.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].deviceProductCode",
		"fdaDevices[].deviceProductCode",
		String,
		["FDA.G.k.12.r.6.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].manufacturerName",
		"fdaDevices[].manufacturerName",
		String,
		["FDA.G.k.12.r.7.1a.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].manufacturerAddress",
		"fdaDevices[].manufacturerAddress",
		String,
		["FDA.G.k.12.r.7.1b.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].manufacturerCity",
		"fdaDevices[].manufacturerCity",
		String,
		["FDA.G.k.12.r.7.1c.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].manufacturerState",
		"fdaDevices[].manufacturerState",
		String,
		["FDA.G.k.12.r.7.1d.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].manufacturerCountry",
		"fdaDevices[].manufacturerCountry",
		String,
		["FDA.G.k.12.r.7.1e.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].deviceUsage",
		"fdaDevices[].deviceUsage",
		String,
		["FDA.G.k.12.r.8.LENGTH.MAX", "FDA.G.k.12.r.8.ALLOWED.VALUE"]
	),
	binding!(
		"drugs[].fdaDevices[].deviceLotNumber",
		"fdaDevices[].deviceLotNumber",
		String,
		["FDA.G.k.12.r.9.LENGTH.MAX"]
	),
	binding!(
		"drugs[].fdaDevices[].operatorOfDevice",
		"fdaDevices[].operatorOfDevice",
		String,
		[
			"FDA.G.k.12.r.10.LENGTH.MAX",
			"FDA.G.k.12.r.10.ALLOWED.VALUE"
		]
	),
	binding!(
		"drugs[].fdaDevices[].remedialActions[].valueCode",
		"fdaDevices[].remedialActions[].valueCode",
		String,
		["FDA.G.k.12.r.11.r.LENGTH.MAX"]
	),
];

pub(super) const EXCLUSIONS: &[PortableBindingExclusion] = &[
	PortableBindingExclusion {
		rule_code: "FDA.G.k.10a.NULLFLAVOR.ALLOWED",
		reason: "not_in_case_editor_model",
	},
	PortableBindingExclusion {
		rule_code: "FDA.G.k.12.r.4.NULLFLAVOR.ALLOWED",
		reason: "not_in_case_editor_model",
	},
	PortableBindingExclusion {
		rule_code: "FDA.G.k.12.r.5.NULLFLAVOR.ALLOWED",
		reason: "not_in_case_editor_model",
	},
	PortableBindingExclusion {
		rule_code: "MFDS.G.k.9.i.2.r.2.KR.1.LENGTH.MAX",
		reason: "authority_dependent_business_value",
	},
	PortableBindingExclusion {
		rule_code: "MFDS.G.k.9.i.2.r.3.KR.1.LENGTH.MAX",
		reason: "authority_dependent_business_value",
	},
];
