#!/usr/bin/env python3
"""Seeded black-box fuzzer for persisted Case business-validator rules.

Each scenario proves both edges of one rule: a valid persisted baseline does
not emit the rule, one field mutation emits it, and restoring that field
removes it. Save readback and audit evidence are checked on both mutations.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import sys
import time
import uuid
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from case_editor_input_fuzzer import (
    AUDIT_TABLES,
    NESTED_AUDIT_TABLES,
    audit_key_matches,
    audit_log_complete,
    commit_sha,
    created_row_id,
    get_path,
    object_id,
    redacted,
    response_summary,
    set_path,
    unwrap,
    values_equal,
)
from rbac_rls_blackbox import ApiClient, guard_target


ROOT = Path(__file__).resolve().parents[1]
RULE_RE = re.compile(r'"((?:ICH|FDA|MFDS)\.[A-Za-z0-9_.-]+)"')
INPUT_ONLY_SUFFIXES = (".LENGTH.MAX", ".ALLOWED.VALUE", ".NULLFLAVOR.ALLOWED")
RETAINED_ALLOWED_VALUE_RULES = {
    "ICH.C.1.6.1.r.2.ALLOWED.VALUE",
    "ICH.C.4.r.2.ALLOWED.VALUE",
    "ICH.D.6.NULLFLAVOR.ALLOWED",
    "ICH.D.7.1.r.1a.ALLOWED.VALUE",
    "ICH.D.10.7.1.r.1a.ALLOWED.VALUE",
    "ICH.E.i.2.1a.ALLOWED.VALUE",
    "ICH.F.r.2.2a.ALLOWED.VALUE",
    "ICH.G.k.7.r.2a.ALLOWED.VALUE",
    "ICH.H.3.r.1a.ALLOWED.VALUE",
}
GENERATED_BUSINESS_RULE_CODES = {
    *(f"ICH.E.i.3.2{suffix}.REQUIRED" for suffix in "abcdef"),
    *(
        f"ICH.{section}.r.{suffix}{part}.{kind}"
        for section in ("D.8", "D.10.8")
        for suffix in ("6", "7")
        for part in ("a", "b")
        for kind in ("REQUIRED", "VOCABULARY")
    ),
    *(
        f"ICH.D.9.{group}.r.{field}.{kind}"
        for group in ("2", "4")
        for field, kinds in (("1a", ("REQUIRED", "VOCABULARY")), ("1b", ("REQUIRED", "VOCABULARY")), ("2", ("REQUIRED",)))
        for kind in kinds
    ),
}


@dataclass(frozen=True)
class Scenario:
    ordinal: int
    scenario_id: str
    authority: str
    page: str
    owner: str
    field: str
    projection_field: str
    expected_code: str
    invalid_value: Any
    valid_value: Any
    fixture_values: tuple[tuple[str, Any], ...] = ()


@dataclass
class Event:
    kind: str
    scenario_id: str | None
    scenario_ordinal: int | None
    classification: str
    http_status: int | None
    response: dict[str, Any]


def discover_business_rule_codes(root: Path = ROOT) -> set[str]:
    codes: set[str] = set()
    section_root = root / "crates/libs/validator/src/case/sections"
    for name in "cdefghn":
        path = section_root / f"{name}.rs"
        codes.update(RULE_RE.findall(path.read_text()))
    codes.update(GENERATED_BUSINESS_RULE_CODES)
    return {
        code
        for code in codes
        if not code.endswith((".", ".r"))
        and code != "ICH.E.i.3.2"
        and (
            code in RETAINED_ALLOWED_VALUE_RULES
            or not code.endswith(INPUT_ONLY_SUFFIXES)
        )
    }


def scenario_catalog(seed: int) -> list[Scenario]:
    rng = random.Random(seed)
    year = rng.randint(2020, 2024)
    suffix = f"{rng.getrandbits(32):08x}"
    return [
        Scenario(
            0,
            "c1-received-date-order",
            "ich",
            "CI",
            "safetyReportIdentification",
            "dateOfMostRecentInformation",
            "dateOfMostRecentInformation",
            "ICH.C.1.4.AFTER_C.1.5.FORBIDDEN",
            f"{year}0302",
            f"{year}0304",
        ),
        Scenario(
            1,
            "g-mpid-phpid-exclusive",
            "ich",
            "DG",
            "drug",
            "phpid",
            "phpid",
            "ICH.G.k.2.1.MPID_PHPID.EXCLUSIVE",
            f"PHPID-{suffix}",
            None,
        ),
        Scenario(
            2,
            "g-cumulative-dose-unit-required",
            "ich",
            "DG",
            "drug",
            "cumulativeDoseUnit",
            "cumulative_dose_first_reaction_unit",
            "ICH.G.k.5b.REQUIRED",
            None,
            "mg",
        ),
        Scenario(
            3,
            "g-cumulative-dose-value-required",
            "ich",
            "DG",
            "drug",
            "cumulativeDoseValue",
            "cumulative_dose_first_reaction_value",
            "ICH.G.k.5a.REQUIRED",
            None,
            12.5,
        ),
        Scenario(
            4,
            "g-suspect-drug-aggregate-required",
            "ich",
            "DG",
            "drug",
            "drugCharacterization",
            "drugCharacterization",
            "ICH.G.k.1.AGGREGATE.REQUIRED",
            "2",
            "1",
        ),
        Scenario(
            5,
            "g-authorization-country-required",
            "ich",
            "DG",
            "drug",
            "drugAuthorizationCountry",
            "manufacturer_country",
            "ICH.G.k.3.2.REQUIRED",
            None,
            "KR",
        ),
        Scenario(
            6,
            "c1-report-type-required",
            "ich",
            "CI",
            "safetyReportIdentification",
            "reportType",
            "reportType",
            "ICH.C.1.3.REQUIRED",
            None,
            "1",
        ),
        Scenario(
            7,
            "c1-first-received-required",
            "ich",
            "CI",
            "safetyReportIdentification",
            "dateFirstReceivedFromSource",
            "dateFirstReceivedFromSource",
            "ICH.C.1.4.REQUIRED",
            None,
            f"{year}0303",
        ),
        Scenario(
            8,
            "c1-most-recent-required",
            "ich",
            "CI",
            "safetyReportIdentification",
            "dateOfMostRecentInformation",
            "dateOfMostRecentInformation",
            "ICH.C.1.5.REQUIRED",
            None,
            f"{year}0304",
        ),
        Scenario(
            9,
            "c1-transmission-date-future",
            "ich",
            "CI",
            "safetyReportIdentification",
            "transmissionDate",
            "transmissionDate",
            "ICH.C.1.2.FUTURE_DATE.FORBIDDEN",
            "29990305120000+0900",
            f"{year}0305120000+0900",
        ),
        Scenario(
            10,
            "c1-first-received-future",
            "ich",
            "CI",
            "safetyReportIdentification",
            "dateFirstReceivedFromSource",
            "dateFirstReceivedFromSource",
            "ICH.C.1.4.FUTURE_DATE.FORBIDDEN",
            "29990303",
            f"{year}0303",
        ),
        Scenario(
            11,
            "c1-most-recent-future",
            "ich",
            "CI",
            "safetyReportIdentification",
            "dateOfMostRecentInformation",
            "dateOfMostRecentInformation",
            "ICH.C.1.5.FUTURE_DATE.FORBIDDEN",
            "29990304",
            f"{year}0304",
        ),
        Scenario(
            12,
            "g-active-ingredient-required",
            "ich",
            "DG",
            "drug",
            "mpid",
            "mpid",
            "ICH.G.k.2.3.r.REQUIRED",
            None,
            f"MPID-{suffix}",
        ),
        Scenario(
            13,
            "g-gestation-value-required",
            "ich",
            "DG",
            "drug",
            "gestationPeriodExposureUnit",
            "gestation_period_exposure_unit",
            "ICH.G.k.6a.REQUIRED",
            "week",
            None,
        ),
        Scenario(
            14,
            "g-gestation-unit-required",
            "ich",
            "DG",
            "drug",
            "gestationPeriodExposureValue",
            "gestation_period_exposure_value",
            "ICH.G.k.6b.REQUIRED",
            2,
            None,
        ),
        Scenario(
            15,
            "g-investigational-product-study-only",
            "ich",
            "DG",
            "drug",
            "investigationalProductBlinded",
            "investigational_product_blinded",
            "ICH.G.k.2.5.STUDY.ONLY",
            False,
            None,
        ),
        Scenario(
            16,
            "g-dosage-dose-unit-required",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].doseUnit",
            "dosageInformation.dose_unit",
            "ICH.G.k.4.r.1b.REQUIRED",
            None,
            "mg",
            (("dosageInformation[].doseValue", 2.5),),
        ),
        Scenario(
            17,
            "g-dosage-frequency-unit-required",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].frequencyUnit",
            "dosageInformation.frequency_unit",
            "ICH.G.k.4.r.3.REQUIRED",
            None,
            "d",
            (("dosageInformation[].numberOfUnits", 1),),
        ),
        Scenario(
            18,
            "g-dosage-duration-value-required",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].durationValue",
            "dosageInformation.duration_value",
            "ICH.G.k.4.r.6a.REQUIRED",
            None,
            2,
            (("dosageInformation[].durationUnit", "d"),),
        ),
        Scenario(
            19,
            "g-dosage-duration-unit-required",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].durationUnit",
            "dosageInformation.duration_unit",
            "ICH.G.k.4.r.6b.REQUIRED",
            None,
            "d",
            (("dosageInformation[].durationValue", 2),),
        ),
        Scenario(
            20,
            "g-dose-form-term-version-required",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].doseFormTermIdVersion",
            "dosageInformation.dose_form_termid_version",
            "ICH.G.k.4.r.9.2a.REQUIRED",
            None,
            "1",
            (("dosageInformation[].doseFormTermId", "DF-1"),),
        ),
        Scenario(
            21,
            "g-route-term-version-required",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].routeTermIdVersion",
            "dosageInformation.route_termid_version",
            "ICH.G.k.4.r.10.2a.REQUIRED",
            None,
            "1",
            (("dosageInformation[].routeOfAdministration", "oral"),),
        ),
        Scenario(
            22,
            "g-parent-route-term-version-required",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].parentRouteTermIdVersion",
            "dosageInformation.parent_route_termid_version",
            "ICH.G.k.4.r.11.2a.REQUIRED",
            None,
            "1",
            (("dosageInformation[].parentRouteTermId", "PROUTE-1"),),
        ),
        Scenario(
            23,
            "g-active-substance-name-required",
            "ich",
            "DG",
            "drug",
            "activeSubstances[].substanceName",
            "activeSubstances.substance_name",
            "ICH.G.k.2.3.r.1.REQUIRED",
            None,
            "Fuzz substance",
            (
                ("mpid", None),
                ("activeSubstances[].substanceStrengthValue", 10),
                ("activeSubstances[].substanceStrengthUnit", "mg"),
            ),
        ),
        Scenario(
            24,
            "g-active-substance-term-version-required",
            "ich",
            "DG",
            "drug",
            "activeSubstances[].substanceTermIdVersion",
            "activeSubstances.substance_termid_version",
            "ICH.G.k.2.3.r.2a.REQUIRED",
            None,
            "1",
            (("activeSubstances[].substanceTermId", "SUBSTANCE-1"),),
        ),
        Scenario(
            25,
            "g-active-substance-strength-unit-required",
            "ich",
            "DG",
            "drug",
            "activeSubstances[].substanceStrengthUnit",
            "activeSubstances.strength_unit",
            "ICH.G.k.2.3.r.3b.REQUIRED",
            None,
            "mg",
            (("activeSubstances[].substanceStrengthValue", 10),),
        ),
        Scenario(
            26,
            "g-indication-meddra-version-required",
            "ich",
            "DG",
            "drug",
            "indications[].indicationMeddraVersion",
            "indications.indication_meddra_version",
            "ICH.G.k.7.r.2a.REQUIRED",
            None,
            "26.0",
            (("indications[].indicationMeddraCode", "10000001"),),
        ),
        Scenario(
            27,
            "g-indication-meddra-code-required",
            "ich",
            "DG",
            "drug",
            "indications[].indicationMeddraCode",
            "indications.indication_meddra_code",
            "ICH.G.k.7.r.2b.REQUIRED",
            None,
            "10000001",
            (("indications[].indicationMeddraVersion", "26.0"),),
        ),
        Scenario(
            28,
            "g-dosage-future-date-forbidden",
            "ich",
            "DG",
            "drug",
            "dosageInformation[].firstAdministrationDate",
            "dosageInformation.first_administration_date",
            "ICH.G.k.4.r.4-5.FUTURE_DATE.FORBIDDEN",
            "29990303",
            f"{year}0303",
        ),
        Scenario(29, "e-reaction-language-required", "ich", "AE", "reaction", "reactionLanguage", "reaction_language", "ICH.E.i.1.1b.REQUIRED", None, "eng"),
        Scenario(30, "e-reaction-meddra-version-required", "ich", "AE", "reaction", "reactionMeddraVersionLLT", "reaction_meddra_version", "ICH.E.i.2.1a.REQUIRED", None, "26.0"),
        Scenario(31, "e-reaction-meddra-code-required", "ich", "AE", "reaction", "reactionMeddraCodeLLT", "reaction_meddra_code", "ICH.E.i.2.1b.REQUIRED", None, "10000001"),
        Scenario(32, "e-reaction-future-date-forbidden", "ich", "AE", "reaction", "reactionStartDate", "start_date", "ICH.E.i.4-5.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(33, "e-reaction-duration-value-required", "ich", "AE", "reaction", "reactionDuration.value", "duration_value", "ICH.E.i.6a.REQUIRED", None, 1, (("reactionDuration.unit", "d"),)),
        Scenario(34, "e-reaction-duration-unit-required", "ich", "AE", "reaction", "reactionDuration.unit", "duration_unit", "ICH.E.i.6b.REQUIRED", None, "d", (("reactionDuration.value", 1),)),
        Scenario(35, "e-reaction-outcome-required", "ich", "AE", "reaction", "reactionOutcome", "outcome", "ICH.E.i.7.REQUIRED", None, "1"),
        Scenario(
            36,
            "e-seriousness-criteria-required",
            "ich",
            "AE",
            "reaction",
            "seriousness.serious",
            "serious",
            "ICH.E.i.3.2.CRITERIA.REQUIRED",
            True,
            False,
            tuple((f"seriousness.{field}", None) for field in (
                "criteriaResultsInDeath",
                "criteriaLifeThreatening",
                "criteriaHospitalization",
                "criteriaDisabling",
                "criteriaCongenitalAnomaly",
                "criteriaOtherMedicallyImportant",
            )),
        ),
        Scenario(37, "e-criteria-death-required", "ich", "AE", "reaction", "seriousness.criteriaResultsInDeath", "criteria_death", "ICH.E.i.3.2a.REQUIRED", None, True),
        Scenario(38, "e-criteria-life-threatening-required", "ich", "AE", "reaction", "seriousness.criteriaLifeThreatening", "criteria_life_threatening", "ICH.E.i.3.2b.REQUIRED", None, True),
        Scenario(39, "e-criteria-hospitalization-required", "ich", "AE", "reaction", "seriousness.criteriaHospitalization", "criteria_hospitalization", "ICH.E.i.3.2c.REQUIRED", None, True),
        Scenario(40, "e-criteria-disabling-required", "ich", "AE", "reaction", "seriousness.criteriaDisabling", "criteria_disabling", "ICH.E.i.3.2d.REQUIRED", None, True),
        Scenario(41, "e-criteria-congenital-required", "ich", "AE", "reaction", "seriousness.criteriaCongenitalAnomaly", "criteria_congenital_anomaly", "ICH.E.i.3.2e.REQUIRED", None, True),
        Scenario(42, "e-criteria-medically-important-required", "ich", "AE", "reaction", "seriousness.criteriaOtherMedicallyImportant", "criteria_other_medically_important", "ICH.E.i.3.2f.REQUIRED", None, True),
        Scenario(43, "f-test-date-required", "ich", "LB", "testResult", "testDate", "test_date", "ICH.F.r.1.REQUIRED", None, f"{year}0303"),
        Scenario(44, "f-test-future-date-forbidden", "ich", "LB", "testResult", "testDate", "test_date", "ICH.F.r.1.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(45, "f-test-meddra-version-required", "ich", "LB", "testResult", "testMeddraVersion", "test_meddra_version", "ICH.F.r.2.2a.REQUIRED", None, "26.0"),
        Scenario(46, "f-test-result-unit-required", "ich", "LB", "testResult", "testUnit", "test_result_unit", "ICH.F.r.3.3.REQUIRED", None, "mg/dL"),
        Scenario(47, "f-coded-result-required", "ich", "LB", "testResult", "testResultCode", "test_result_code", "ICH.F.r.3.1.REQUIRED", None, "1", (("testResult", None), ("testResultUnstructured", None))),
        Scenario(48, "f-value-result-required", "ich", "LB", "testResult", "testResult", "test_result_value", "ICH.F.r.3.2.REQUIRED", None, "12.5", (("testResultCode", None), ("testResultUnstructured", None))),
        Scenario(49, "f-unstructured-result-required", "ich", "LB", "testResult", "testResultUnstructured", "result_unstructured", "ICH.F.r.3.4.REQUIRED", None, "Normal", (("testResultCode", None), ("testResult", None))),
        Scenario(50, "d-patient-age-exclusive", "ich", "DM", "patientInformation", "patientAgeGroup", "age_group", "ICH.D.2.EXCLUSIVE", "5", None, (("patientBirthDate", f"{year}0303"),)),
        Scenario(51, "d-patient-birth-date-future", "ich", "DM", "patientInformation", "patientBirthDate", "birth_date", "ICH.D.2.1.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(52, "d-patient-age-value-required", "ich", "DM", "patientInformation", "patientAge.value", "age_at_time_of_onset", "ICH.D.2.2a.REQUIRED", None, 36.5, (("patientAge.unit", "a"),)),
        Scenario(53, "d-patient-age-unit-required", "ich", "DM", "patientInformation", "patientAge.unit", "age_unit", "ICH.D.2.2b.REQUIRED", None, "a", (("patientAge.value", 36.5),)),
        Scenario(54, "d-patient-gestation-value-required", "ich", "DM", "patientInformation", "gestationPeriod.value", "gestation_period", "ICH.D.2.2.1a.REQUIRED", None, 22, (("gestationPeriod.unit", "wk"),)),
        Scenario(55, "d-patient-gestation-unit-required", "ich", "DM", "patientInformation", "gestationPeriod.unit", "gestation_period_unit", "ICH.D.2.2.1b.REQUIRED", None, "wk", (("gestationPeriod.value", 22),)),
        Scenario(56, "d-patient-lmp-future", "ich", "DM", "patientInformation", "lastMenstrualPeriodDate", "last_menstrual_period_date", "ICH.D.6.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(57, "h-diagnosis-meddra-version-required", "ich", "NR", "senderDiagnoses", "diagnosisMeddraVersion", "diagnosis_meddra_version", "ICH.H.3.r.1a.REQUIRED", None, "26.0"),
        Scenario(58, "h-diagnosis-meddra-code-required", "ich", "NR", "senderDiagnoses", "diagnosisMeddraCode", "diagnosis_meddra_code", "ICH.H.3.r.1b.REQUIRED", None, "10000001"),
        Scenario(59, "h-case-summary-language-required", "ich", "NR", "caseSummaryInformation", "languageCode", "language_code", "ICH.H.5.r.1b.REQUIRED", None, "eng"),
        Scenario(60, "d-medical-history-text-required", "ich", "DM", "patientInformation", "medicalHistoryText", "medical_history_text", "ICH.D.7.2.REQUIRED", None, "No relevant history"),
        Scenario(61, "d-history-meddra-version-required", "ich", "DM", "medicalHistoryEpisodes", "meddraVersion", "meddra_version", "ICH.D.7.1.r.1a.REQUIRED", None, "26.0"),
        Scenario(62, "d-history-meddra-code-required", "ich", "DM", "medicalHistoryEpisodes", "meddraCode", "meddra_code", "ICH.D.7.1.r.1b.REQUIRED", None, "10000001"),
        Scenario(63, "d-history-future-date-forbidden", "ich", "DM", "medicalHistoryEpisodes", "startDate", "start_date", "ICH.D.7.1.r.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(64, "d-past-drug-name-required", "ich", "DH", "pastDrugHistory", "drugName", "drug_name", "ICH.D.8.r.1.REQUIRED", None, "Prior drug"),
        Scenario(65, "d-past-drug-identifier-exclusive", "ich", "DH", "pastDrugHistory", "phpid", "phpid", "ICH.D.8.MPID_PHPID.EXCLUSIVE", f"PHPID-{suffix}", None),
        Scenario(66, "d-past-drug-future-date-forbidden", "ich", "DH", "pastDrugHistory", "startDate", "start_date", "ICH.D.8.r.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(67, "d-past-drug-indication-version-required", "ich", "DH", "pastDrugHistory", "indicationMeddraVersion", "indication_meddra_version", "ICH.D.8.r.6a.REQUIRED", None, "26.0"),
        Scenario(68, "d-past-drug-indication-code-required", "ich", "DH", "pastDrugHistory", "indicationMeddraCode", "indication_meddra_code", "ICH.D.8.r.6b.REQUIRED", None, "10000001"),
        Scenario(69, "d-past-drug-reaction-version-required", "ich", "DH", "pastDrugHistory", "reactionMeddraVersion", "reaction_meddra_version", "ICH.D.8.r.7a.REQUIRED", None, "26.0"),
        Scenario(70, "d-past-drug-reaction-code-required", "ich", "DH", "pastDrugHistory", "reactionMeddraCode", "reaction_meddra_code", "ICH.D.8.r.7b.REQUIRED", None, "10000001"),
        Scenario(71, "d-death-date-future", "ich", "DM", "deathInfo", "dateOfDeath", "date_of_death", "ICH.D.9.1.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(72, "d-autopsy-performed-required", "ich", "DM", "deathInfo", "autopsyPerformed", "autopsy_performed", "ICH.D.9.3.REQUIRED", None, True, (("dateOfDeath", f"{year}0303"),)),
        Scenario(73, "d-reported-cause-version-required", "ich", "DM", "reportedCauses", "meddraVersion", "meddra_version", "ICH.D.9.2.r.1a.REQUIRED", None, "26.0"),
        Scenario(74, "d-reported-cause-code-required", "ich", "DM", "reportedCauses", "meddraCode", "meddra_code", "ICH.D.9.2.r.1b.REQUIRED", None, "10000001"),
        Scenario(75, "d-reported-cause-text-required", "ich", "DM", "reportedCauses", "causeText", "comments", "ICH.D.9.2.r.2.REQUIRED", None, "Reported cause"),
        Scenario(76, "d-autopsy-cause-version-required", "ich", "DM", "autopsyCauses", "meddraVersion", "meddra_version", "ICH.D.9.4.r.1a.REQUIRED", None, "26.0"),
        Scenario(77, "d-autopsy-cause-code-required", "ich", "DM", "autopsyCauses", "meddraCode", "meddra_code", "ICH.D.9.4.r.1b.REQUIRED", None, "10000001"),
        Scenario(78, "d-autopsy-cause-text-required", "ich", "DM", "autopsyCauses", "causeText", "comments", "ICH.D.9.4.r.2.REQUIRED", None, "Autopsy cause"),
        Scenario(79, "d-parent-birth-date-future", "ich", "DM", "parentInfo", "parentBirthDate", "parent_birth_date", "ICH.D.10.2.1.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(80, "d-parent-age-value-required", "ich", "DM", "parentInfo", "parentAge.value", "parent_age", "ICH.D.10.2.2a.REQUIRED", None, 54, (("parentAge.unit", "a"),)),
        Scenario(81, "d-parent-age-unit-required", "ich", "DM", "parentInfo", "parentAge.unit", "parent_age_unit", "ICH.D.10.2.2b.REQUIRED", None, "a", (("parentAge.value", 54),)),
        Scenario(82, "d-parent-birth-age-exclusive", "ich", "DM", "parentInfo", "parentBirthDate", "parent_birth_date", "ICH.D.10.2.EXCLUSIVE", f"{year}0303", None, (("parentAge.value", 54), ("parentAge.unit", "a"))),
        Scenario(83, "d-parent-lmp-future", "ich", "DM", "parentInfo", "parentLastMenstrualPeriodDate", "last_menstrual_period_date", "ICH.D.10.3.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(84, "d-parent-sex-required", "ich", "DM", "parentInfo", "parentSex", "sex", "ICH.D.10.6.REQUIRED", None, "2", (("parentIdentification", "FUZZ-PARENT"),)),
        Scenario(85, "d-parent-history-version-required", "ich", "DM", "parentMedicalHistory", "meddraVersion", "meddra_version", "ICH.D.10.7.1.r.1a.REQUIRED", None, "26.0"),
        Scenario(86, "d-parent-history-code-required", "ich", "DM", "parentMedicalHistory", "meddraCode", "meddra_code", "ICH.D.10.7.1.r.1b.REQUIRED", None, "10000001"),
        Scenario(87, "d-parent-history-future-date", "ich", "DM", "parentMedicalHistory", "startDate", "start_date", "ICH.D.10.7.1.r.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(88, "d-parent-past-drug-mpid-version-required", "ich", "DM", "parentPastDrugs", "mpidVersion", "mpid_version", "ICH.D.10.8.r.2a.REQUIRED", None, "1"),
        Scenario(89, "d-parent-past-drug-phpid-version-required", "ich", "DM", "parentPastDrugs", "phpidVersion", "phpid_version", "ICH.D.10.8.r.3a.REQUIRED", None, "1", (("mpid", None), ("mpidVersion", None), ("phpid", f"PHPID-{suffix}"))),
        Scenario(90, "d-parent-past-drug-future-date", "ich", "DM", "parentPastDrugs", "startDate", "start_date", "ICH.D.10.8.r.FUTURE_DATE.FORBIDDEN", "29990303", f"{year}0303"),
        Scenario(91, "d-parent-past-drug-identifier-exclusive", "ich", "DM", "parentPastDrugs", "phpid", "phpid", "ICH.D.10.8.MPID_PHPID.EXCLUSIVE", f"PHPID-{suffix}", None, (("phpidVersion", "1"),)),
        Scenario(92, "d-parent-past-drug-indication-version-required", "ich", "DM", "parentPastDrugs", "indicationMeddraVersion", "indication_meddra_version", "ICH.D.10.8.r.6a.REQUIRED", None, "26.0"),
        Scenario(93, "d-parent-past-drug-indication-code-required", "ich", "DM", "parentPastDrugs", "indicationMeddraCode", "indication_meddra_code", "ICH.D.10.8.r.6b.REQUIRED", None, "10000001"),
        Scenario(94, "d-parent-past-drug-reaction-version-required", "ich", "DM", "parentPastDrugs", "reactionMeddraVersion", "reaction_meddra_version", "ICH.D.10.8.r.7a.REQUIRED", None, "26.0"),
        Scenario(95, "d-parent-past-drug-reaction-code-required", "ich", "DM", "parentPastDrugs", "reactionMeddraCode", "reaction_meddra_code", "ICH.D.10.8.r.7b.REQUIRED", None, "10000001"),
        Scenario(96, "d-history-parent-duplicate", "ich", "DM", "medicalHistoryEpisodes", "familyHistory", "family_history", "ICH.D.7.1.r.6.PARENT_DUPLICATE", True, False),
        Scenario(97, "c1-expedited-criteria-required", "ich", "CI", "safetyReportIdentification", "fulfilExpeditedCriteria", "fulfilExpeditedCriteria", "ICH.C.1.7.REQUIRED", None, True),
        Scenario(98, "c1-other-identifiers-flag-required", "ich", "CI", "safetyReportIdentification", "otherCaseIdentifiersExist", "otherCaseIdentifiersExist", "ICH.C.1.9.1.REQUIRED", None, True),
        Scenario(99, "c1-nullification-reason-required", "ich", "CI", "safetyReportIdentification", "nullificationReason", "nullificationReason", "ICH.C.1.11.2.REQUIRED", None, "Amendment reason", (("nullificationAmendmentCode", "2"),)),
        Scenario(100, "g-assessment-administration-value-required", "ich", "DG", "drug", "drugReactionAssessments[].administrationStartIntervalValue", "drugReactionAssessments[].administrationStartIntervalValue", "ICH.G.k.9.i.3.1a.REQUIRED", None, 1.5, (("drugReactionAssessments[].administrationStartIntervalUnit", "d"),)),
        Scenario(101, "mfds-c1-additional-documents-required", "mfds", "CI", "safetyReportIdentification", "additionalDocumentsAvailable", "additionalDocumentsAvailable", "ICH.C.1.6.1.REQUIRED", None, False),
        Scenario(102, "mfds-c1-worldwide-id-required", "mfds", "CI", "safetyReportIdentification", "worldwideUniqueId", "worldwideUniqueId", "ICH.C.1.8.1.REQUIRED", None, f"KR-BUSINESS-{suffix}"),
        Scenario(103, "mfds-c1-first-sender-required", "mfds", "CI", "safetyReportIdentification", "firstSenderType", "firstSenderType", "ICH.C.1.8.2.REQUIRED", None, "1"),
        Scenario(104, "c2-reporter-country-required", "ich", "RP", "primarySources", "reporterCountry", "reporterCountry", "ICH.C.2.r.3.REQUIRED", None, "KR"),
        Scenario(105, "c2-reporter-qualification-required", "ich", "RP", "primarySources", "qualification", "qualification", "ICH.C.2.r.4.REQUIRED", None, "1"),
        Scenario(106, "c2-primary-source-required", "ich", "RP", "primarySources", "primarySourceForRegulatoryPurposes", "primarySourceForRegulatoryPurposes", "ICH.C.2.r.5.REQUIRED", None, "1"),
        Scenario(107, "c2-primary-source-exactly-once", "ich", "RP", "primarySources", "primarySourceForRegulatoryPurposes", "primarySourceForRegulatoryPurposes", "ICH.C.2.r.5.EXACTLY_ONCE", "1", None),
        Scenario(108, "c3-sender-type-required", "ich", "SD", "senderInformation", "senderType", "senderType", "ICH.C.3.1.REQUIRED", None, "1"),
        Scenario(109, "c3-sender-organization-required", "ich", "SD", "senderInformation", "organizationName", "organizationName", "ICH.C.3.2.REQUIRED", None, "Business Sender"),
        Scenario(110, "c5-sponsor-study-number-required", "ich", "SI", "studyInformation", "sponsorStudyNumber", "sponsor_study_number", "ICH.C.5.3.REQUIRED", None, f"STUDY-{suffix}"),
        Scenario(111, "c5-study-type-reaction-required", "ich", "SI", "studyInformation", "studyTypeReaction", "study_type_reaction", "ICH.C.5.4.REQUIRED", None, "1"),
        Scenario(112, "mfds-d-past-drug-mpid-version-required", "mfds", "DH", "pastDrugHistory", "mpidVersion", "mpid_version", "MFDS.D.8.r.2a.REQUIRED", None, "1"),
        Scenario(113, "mfds-d-past-drug-mpid-required", "mfds", "DH", "pastDrugHistory", "mpid", "mpid", "MFDS.D.8.r.2b.REQUIRED", None, f"MPID-{suffix}", (("mpidVersion", "1"),)),
        Scenario(114, "mfds-d-past-drug-phpid-version-required", "mfds", "DH", "pastDrugHistory", "phpidVersion", "phpid_version", "MFDS.D.8.r.3a.REQUIRED", None, "1", (("mpid", None), ("phpid", f"PHPID-{suffix}"))),
        Scenario(115, "mfds-d-past-drug-phpid-required", "mfds", "DH", "pastDrugHistory", "phpid", "phpid", "MFDS.D.8.r.3b.REQUIRED", None, f"PHPID-{suffix}", (("mpid", None), ("mpidVersion", None), ("phpidVersion", "1"))),
        Scenario(116, "mfds-d-parent-past-drug-mpid-required", "mfds", "DM", "parentPastDrugs", "mpid", "mpid", "MFDS.D.10.8.r.2b.REQUIRED", None, f"MPID-{suffix}", (("mpidVersion", "1"),)),
        Scenario(117, "mfds-d-parent-past-drug-phpid-required", "mfds", "DM", "parentPastDrugs", "phpid", "phpid", "MFDS.D.10.8.r.3b.REQUIRED", None, f"PHPID-{suffix}", (("mpid", None), ("mpidVersion", None), ("phpidVersion", "1"))),
        Scenario(118, "g-assessment-administration-unit-required", "ich", "DG", "drug", "drugReactionAssessments[].administrationStartIntervalUnit", "drugReactionAssessments[].administrationStartIntervalUnit", "ICH.G.k.9.i.3.1b.REQUIRED", None, "d", (("drugReactionAssessments[].administrationStartIntervalValue", 1.5),)),
        Scenario(119, "g-assessment-last-dose-value-required", "ich", "DG", "drug", "drugReactionAssessments[].lastDoseIntervalValue", "drugReactionAssessments[].lastDoseIntervalValue", "ICH.G.k.9.i.3.2a.REQUIRED", None, 2.5, (("drugReactionAssessments[].lastDoseIntervalUnit", "d"),)),
        Scenario(120, "g-assessment-last-dose-unit-required", "ich", "DG", "drug", "drugReactionAssessments[].lastDoseIntervalUnit", "drugReactionAssessments[].lastDoseIntervalUnit", "ICH.G.k.9.i.3.2b.REQUIRED", None, "d", (("drugReactionAssessments[].lastDoseIntervalValue", 2.5),)),
        Scenario(121, "mfds-reaction-eu-country-forbidden", "mfds", "AE", "reaction", "reactionCountry", "country_code", "MFDS.E.i.9.EU.FORBIDDEN", "EU", "KR"),
        Scenario(122, "fda-required-intervention-required", "fda", "AE", "reaction", "requiredIntervention", "required_intervention", "FDA.E.i.3.2h.REQUIRED", None, True),
        Scenario(123, "reaction-hcp-medical-confirmation-omit", "ich", "AE", "reaction", "medicalConfirmation", "medical_confirmation", "ICH.E.i.8.HCP.OMIT", True, None),
        Scenario(124, "mfds-more-test-info-documents-required", "mfds", "LB", "testResult", "moreInformationAvailable", "more_info_available", "MFDS.F.r.7.C.1.6.1.REQUIRED", True, False),
        Scenario(125, "mfds-drug-mpid-version-required", "mfds", "DG", "drug", "mpidVersion", "mpid_version", "MFDS.G.k.2.1.1a.REQUIRED", None, "1"),
        Scenario(126, "mfds-drug-mpid-required", "mfds", "DG", "drug", "mpid", "mpid", "MFDS.G.k.2.1.1b.REQUIRED", None, f"MPID-{suffix}", (("mpidVersion", "1"),)),
        Scenario(127, "mfds-drug-phpid-version-required", "mfds", "DG", "drug", "phpidVersion", "phpid_version", "MFDS.G.k.2.1.2a.REQUIRED", None, "1", (("phpid", f"PHPID-{suffix}"),)),
        Scenario(128, "mfds-drug-phpid-required", "mfds", "DG", "drug", "phpid", "phpid", "MFDS.G.k.2.1.2b.REQUIRED", None, f"PHPID-{suffix}", (("phpidVersion", "1"),)),
        Scenario(129, "mfds-substance-term-id-required", "mfds", "DG", "drug", "activeSubstances[].substanceTermId", "activeSubstances[].substance_termid", "MFDS.G.k.2.3.r.2b.REQUIRED", None, f"SUB-{suffix}", (("activeSubstances[].substanceTermIdVersion", "1"),)),
        Scenario(130, "mfds-domestic-product-code-required", "mfds", "DG", "drug", "mfdsMpid", "mfds_mpid", "MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED", None, "KR12345678", (("obtainDrugCountry", "KR"),)),
        Scenario(131, "mfds-foreign-whompid-required", "mfds", "DG", "drug", "mfdsMpid", "mfds_mpid", "MFDS.KR.FOREIGN.WHOMPID.REQUIRED", None, "WH12345678", (("obtainDrugCountry", "US"),)),
        Scenario(132, "mfds-domestic-ingredient-code-required", "mfds", "DG", "drug", "activeSubstances[].mfdsId", "activeSubstances[].mfds_id", "MFDS.KR.DOMESTIC.INGREDIENTCODE.REQUIRED", None, "KS12345678", (("obtainDrugCountry", "KR"), ("activeSubstances[].substanceName", "Business ingredient"))),
        Scenario(133, "c1-document-description-required", "ich", "CI", "documentsHeldBySender", "documentDescription", "documentDescription", "ICH.C.1.6.1.r.1.REQUIRED", None, "Clinical evidence", (("includedDocument", "SGVsbG8="),)),
        Scenario(134, "c1-document-base64-required", "ich", "CI", "documentsHeldBySender", "includedDocument", "includedDocument", "ICH.C.1.6.1.r.2.ALLOWED.VALUE", "not-base64", "SGVsbG8=", (("documentDescription", "Clinical evidence"),)),
        Scenario(135, "fda-document-file-name-required", "fda", "CI", "documentsHeldBySender", "fileName", "file_name", "FDA.C.1.6.1.r.2.FILE_NAME.REQUIRED", None, "evidence.pdf", (("documentDescription", "Clinical evidence"), ("includedDocument", "SGVsbG8="), ("mediaType", "application/pdf"))),
        Scenario(136, "fda-document-media-type-match", "fda", "CI", "documentsHeldBySender", "mediaType", "media_type", "FDA.C.1.6.1.r.2.MEDIA_TYPE.MATCH", "text/plain", "application/pdf", (("documentDescription", "Clinical evidence"), ("includedDocument", "SGVsbG8="), ("fileName", "evidence.pdf"))),
        Scenario(137, "c2-study-reporter-organization-required", "ich", "RP", "primarySources", "reporterOrganization", "reporterOrganization", "ICH.C.2.r.2.1.REQUIRED", None, "Business Reporter"),
        Scenario(138, "fda-combination-indicator-required", "fda", "CI", "safetyReportIdentification", "combinationProductReportIndicator", "combinationProductReportIndicator", "FDA.C.1.12.REQUIRED", None, "false"),
        Scenario(139, "fda-combination-indicator-recommended", "fda", "CI", "safetyReportIdentification", "combinationProductReportIndicator", "combinationProductReportIndicator", "FDA.C.1.12.RECOMMENDED", None, "false"),
        Scenario(140, "fda-local-criteria-required", "fda", "CI", "safetyReportIdentification", "localCriteriaReportType", "localCriteriaReportType", "FDA.C.1.7.1.REQUIRED", None, "1"),
        Scenario(141, "fda-documents-flag-row-required", "fda", "CI", "safetyReportIdentification", "additionalDocumentsAvailable", "additionalDocumentsAvailable", "FDA.R0009", True, False),
        Scenario(142, "fda-identifiers-flag-row-required", "fda", "CI", "safetyReportIdentification", "otherCaseIdentifiersExist", "otherCaseIdentifiersExist", "FDA.R0017", True, False),
        Scenario(143, "fda-primary-qualification-required", "fda", "RP", "primarySources", "qualification", "qualification", "FDA.R0020", None, "1"),
        Scenario(144, "fda-sender-contact-required", "fda", "SD", "senderInformation", "email", "email", "FDA.C.3.SENDER.REQUIRED", None, "sender@example.com"),
    ]


def issue_codes(value: Any) -> set[str]:
    issues = value.get("issues", []) if isinstance(value, dict) else []
    return {
        issue["code"]
        for issue in issues
        if isinstance(issue, dict) and isinstance(issue.get("code"), str)
    }


def issue_complete(value: Any, code: str) -> bool:
    issues = value.get("issues", []) if isinstance(value, dict) else []
    return any(
        isinstance(issue, dict)
        and issue.get("code") == code
        and isinstance(issue.get("message"), str)
        and bool(issue.get("message"))
        and isinstance(issue.get("path"), str)
        and bool(issue.get("path"))
        and isinstance(issue.get("section"), str)
        and isinstance(issue.get("subsection"), str)
        and isinstance(issue.get("blocking"), bool)
        for issue in issues
    )


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--base-url", default=os.getenv("E2BR3_BASE_URL", "http://127.0.0.1:8080"))
    result.add_argument("--email", default=os.getenv("E2BR3_ADMIN_EMAIL", "demo.cro.admin@example.com"))
    result.add_argument("--password", default=os.getenv("E2BR3_ADMIN_PASSWORD", "welcome"))
    result.add_argument("--seed", type=int, default=int(time.time()))
    result.add_argument("--artifact-dir", default="tmp/rbac-rls-fuzz/business-validator")
    result.add_argument("--timeout", type=float, default=20)
    result.add_argument("--max-actions", type=int, default=500)
    result.add_argument("--deadline-seconds", type=float, default=300)
    result.add_argument("--scenario", action="append", help="run only this scenario id (repeatable)")
    result.add_argument("--allow-remote", action="store_true")
    result.add_argument("--dry-run", action="store_true")
    return result


def main(args: argparse.Namespace) -> int:
    guard_target(args.base_url, args.allow_remote)
    scenarios = scenario_catalog(args.seed)
    if args.scenario:
        requested = set(args.scenario)
        unknown = requested - {scenario.scenario_id for scenario in scenarios}
        if unknown:
            raise SystemExit(f"unknown scenario ids: {', '.join(sorted(unknown))}")
        scenarios = [scenario for scenario in scenarios if scenario.scenario_id in requested]
    inventory = discover_business_rule_codes()
    covered = {scenario.expected_code for scenario in scenarios}
    if args.dry_run:
        print(json.dumps({
            "seed": args.seed,
            "scenarios": len(scenarios),
            "covered_rules": len(covered),
            "inventory_rules": len(inventory),
            "uncovered_rules": sorted(inventory - covered),
        }, sort_keys=True))
        return 0
    if not args.password:
        raise SystemExit("set E2BR3_ADMIN_PASSWORD")

    client = ApiClient(args.base_url, args.timeout)
    events: list[Event] = []
    started = time.monotonic()
    requests = 0
    interrupted: str | None = None

    def add(kind: str, scenario: Scenario | None, classification: str, status: int | None, detail: dict[str, Any]) -> None:
        events.append(Event(
            kind,
            scenario.scenario_id if scenario else None,
            scenario.ordinal if scenario else None,
            classification,
            status,
            detail,
        ))

    def request(method: str, path: str, payload: dict[str, Any] | None = None) -> tuple[int | None, Any, dict[str, Any]]:
        nonlocal interrupted, requests
        if requests >= args.max_actions:
            interrupted = interrupted or "max_actions"
            return None, None, {}
        if time.monotonic() - started >= args.deadline_seconds:
            interrupted = interrupted or "deadline"
            return None, None, {}
        requests += 1
        status, body, transport = client.request(method, path, payload)
        summary = response_summary(status, body)
        if transport:
            summary["transport_error"] = transport
            interrupted = interrupted or "transport_error"
        if status is not None and status >= 500:
            interrupted = interrupted or "server_error"
        try:
            value = unwrap(json.loads(body))
        except (UnicodeDecodeError, json.JSONDecodeError):
            value = None
        return status, value, summary

    def page_current(case_id: str, page: str, owner: str, row_id: str | None = None) -> tuple[int | None, Any]:
        route = f"/api/cases/{case_id}/editor/pages/{page}"
        status, value, _ = request("GET", f"{route}/rows/{row_id}" if row_id else route)
        if row_id:
            return status, value.get(owner, value) if isinstance(value, dict) else value
        rows = value.get("rows", {}) if isinstance(value, dict) else {}
        current = rows.get(owner) if isinstance(rows, dict) else None
        if isinstance(current, list):
            current = current[0] if current else None
        return status, current

    def audit_logs(case_id: str, owner: str, row_id: str, field: str) -> list[dict[str, Any]]:
        nested_table = next(
            (table for prefix, table in NESTED_AUDIT_TABLES.items() if field.startswith(prefix)),
            None,
        )
        if nested_table:
            status, value, _ = request("GET", f"/api/audit-logs/by-record/cases/{case_id}")
            return [
                item for item in value
                if isinstance(item, dict) and item.get("table_name") == nested_table
            ] if status == 200 and isinstance(value, list) else []
        table = AUDIT_TABLES.get(owner)
        target = f"{table}/{row_id}" if table else f"cases/{case_id}"
        status, value, _ = request("GET", f"/api/audit-logs/by-record/{target}")
        if status != 200:
            return []
        if isinstance(value, list):
            return [item for item in value if isinstance(item, dict)]
        items = value.get("items", value.get("data", [])) if isinstance(value, dict) else []
        return [item for item in items if isinstance(item, dict)] if isinstance(items, list) else []

    def validation(case_id: str, authority: str) -> tuple[int | None, Any, dict[str, Any]]:
        return request("GET", f"/api/cases/{case_id}/validation?authority={authority}")

    status, _, summary = request("POST", "/auth/v1/login", {"email": args.email, "pwd": args.password})
    add("login", None, "PASS" if status == 200 else "FAIL", status, summary)
    if status != 200:
        interrupted = interrupted or "login_failed"

    year = scenario_catalog(args.seed)[0].invalid_value[:4]

    def create_case() -> tuple[int | None, str | None, dict[str, Any]]:
        status, value, summary = request("POST", "/api/cases", {
            "data": {
                "safetyReportIdentification": {"safetyReportId": f"BUSINESS-FUZZ-{uuid.uuid4()}"},
                "status": "draft",
            }
        })
        return status, object_id(value), summary

    def ci_payload(
        field: str | None = None,
        value: Any = None,
        fixture_scenario: Scenario | None = None,
    ) -> dict[str, Any]:
        payload = {
            "transmissionDate": f"{year}0305120000+0900",
            "reportType": "1",
            "dateFirstReceivedFromSource": f"{year}0303",
            "dateOfMostRecentInformation": f"{year}0304",
        }
        if fixture_scenario:
            for path, fixture_value in fixture_scenario.fixture_values:
                set_path(payload, path, fixture_value)
        if field:
            if value is None:
                payload.pop(field, None)
            else:
                payload[field] = value
        return payload

    def drug_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "drugCharacterization": "1",
            "medicinalProduct": "Business fuzz product",
            "mpid": f"MPID-{args.seed}",
            "cumulativeDoseValue": 12.5,
            "cumulativeDoseUnit": "mg",
            "drugAuthorizationNumber": f"AUTH-{args.seed}",
            "drugAuthorizationCountry": "KR",
        }
        for path, fixture_value in scenario.fixture_values:
            if fixture_value is None and "." not in path:
                payload.pop(path, None)
            else:
                set_path(payload, path, fixture_value)
        if value is None and "." not in scenario.field:
            payload.pop(scenario.field, None)
        elif value is not None:
            set_path(payload, scenario.field, value)
        return payload

    def reaction_payload(scenario: Scenario | None = None, value: Any = None) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "sequenceNumber": 1,
            "primarySourceReaction": "Business fuzz reaction",
            "reactionLanguage": "eng",
            "reactionMeddraVersionLLT": "26.0",
            "reactionMeddraCodeLLT": "10000001",
            "reactionStartDate": f"{year}0303",
            "reactionEndDate": f"{year}0304",
            "reactionDuration": {"value": 1, "unit": "d"},
            "reactionOutcome": "1",
            "seriousness": {
                "serious": True,
                "criteriaResultsInDeath": True,
                "criteriaLifeThreatening": True,
                "criteriaHospitalization": True,
                "criteriaDisabling": True,
                "criteriaCongenitalAnomaly": True,
                "criteriaOtherMedicallyImportant": True,
            },
        }
        if scenario:
            for path, fixture_value in scenario.fixture_values:
                set_path(payload, path, fixture_value)
            set_path(payload, scenario.field, value)
        return payload

    def test_result_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "sequenceNumber": 1,
            "testDate": f"{year}0303",
            "testName": "ALT",
            "testMeddraVersion": "26.0",
            "testMeddraCode": "10000001",
            "testResultCode": "1",
            "testResult": "12.5",
            "testUnit": "mg/dL",
            "testResultUnstructured": "Normal",
        }
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def patient_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {"patientInitials": "BUSINESS-FUZZ"}
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def medical_history_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "sequenceNumber": 1,
            "meddraVersion": "26.0",
            "meddraCode": "10000001",
            "startDate": f"{year}0303",
            "endDate": f"{year}0304",
            "continuing": True,
        }
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def death_info_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {"autopsyPerformed": True}
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def death_cause_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "meddraVersion": "26.0",
            "meddraCode": "10000001",
            "causeText": "Business fuzz cause",
        }
        set_path(payload, scenario.field, value)
        return payload

    def parent_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {"parentSex": "2"}
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def parent_history_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "meddraVersion": "26.0",
            "meddraCode": "10000001",
            "startDate": f"{year}0303",
            "endDate": f"{year}0304",
            "continuing": True,
        }
        set_path(payload, scenario.field, value)
        return payload

    def parent_past_drug_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "drugName": "Parent prior drug",
            "mpidVersion": "1",
            "mpid": f"MPID-{args.seed}",
            "startDate": f"{year}0303",
            "endDate": f"{year}0304",
            "indicationMeddraVersion": "26.0",
            "indicationMeddraCode": "10000001",
            "reactionMeddraVersion": "26.0",
            "reactionMeddraCode": "10000001",
        }
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def past_drug_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "sequenceNumber": 1,
            "drugName": "Prior drug",
            "mpid": f"MPID-{args.seed}",
            "startDate": f"{year}0303",
            "endDate": f"{year}0304",
            "indicationMeddraVersion": "26.0",
            "indicationMeddraCode": "10000001",
            "reactionMeddraVersion": "26.0",
            "reactionMeddraCode": "10000001",
        }
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def narrative_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        if scenario.owner == "senderDiagnoses":
            payload: dict[str, Any] = {
                "sequenceNumber": 1,
                "diagnosisMeddraVersion": "26.0",
                "diagnosisMeddraCode": "10000001",
            }
        else:
            payload = {"sequenceNumber": 1, "summaryText": "Business fuzz summary", "languageCode": "eng"}
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def primary_source_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "reporterOrganization": "Business Reporter",
            "reporterCountry": "KR",
            "qualification": "1",
            "primarySourceForRegulatoryPurposes": "1",
        }
        set_path(payload, scenario.field, value)
        return payload

    def document_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {}
        for path, fixture_value in scenario.fixture_values:
            set_path(payload, path, fixture_value)
        set_path(payload, scenario.field, value)
        return payload

    def sender_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "senderType": "1",
            "organizationName": "Business Sender",
            "department": "Safety",
            "personTitle": "Dr",
            "personGivenName": "Business",
            "personFamilyName": "Sender",
            "streetAddress": "1 Test Street",
            "city": "Seoul",
            "state": "Seoul",
            "postcode": "04524",
            "countryCode": "KR",
            "telephone": "+82-2-1234-5678",
            "fax": "+82-2-1234-5679",
            "email": "sender@example.com",
        }
        set_path(payload, scenario.field, value)
        return payload

    def study_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "studyName": "Business Study",
            "sponsorStudyNumber": f"STUDY-{args.seed}",
            "studyTypeReaction": "1",
        }
        set_path(payload, scenario.field, value)
        return payload

    def run_edge(scenario: Scenario, edge: str, value: Any) -> None:
        nonlocal interrupted
        status, case_id, summary = create_case()
        if status != 201 or not case_id:
            add(edge, scenario, "FAIL", status, {**summary, "reason": "case_create_failed"})
            interrupted = interrupted or "case_create_failed"
            return

        status, current = page_current(case_id, "CI", "safetyReportIdentification")
        ci_id = object_id(current)
        ci = ci_payload(scenario.field, value, scenario) if scenario.owner == "safetyReportIdentification" else ci_payload()
        if scenario.page == "SI" or scenario.scenario_id == "c2-study-reporter-organization-required":
            ci["reportType"] = "2"
        ci["id"] = ci_id
        status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/CI", {
            "authorities": ["ich"],
            "rows": {"safetyReportIdentification": ci},
        })
        if status != 200 or not ci_id:
            add(edge, scenario, "FAIL", status, {**save_summary, "reason": "ci_fixture_failed"})
            return

        if scenario.scenario_id == "reaction-hcp-medical-confirmation-omit":
            status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/RP", {
                "authorities": [scenario.authority],
                "rows": {"primarySources": [{
                    "reporterOrganization": "Business HCP",
                    "reporterCountry": "KR",
                    "qualification": "1",
                    "primarySourceForRegulatoryPurposes": "1",
                }]},
            })
            if status != 200:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "hcp_fixture_failed"})
                return

        if scenario.page == "CI":
            if scenario.owner == "safetyReportIdentification":
                owner_id = ci_id
                read_status, current = page_current(case_id, "CI", scenario.owner)
            else:
                status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/CI", {
                    "authorities": [scenario.authority],
                    "rows": {scenario.owner: [document_payload(scenario, value)]},
                })
                read_status, current = page_current(case_id, "CI", scenario.owner)
                owner_id = object_id(current)
                if status != 200 or not owner_id:
                    add(edge, scenario, "FAIL", status, {**save_summary, "reason": "ci_document_fixture_failed"})
                    return
        elif scenario.page == "RP":
            sources = [primary_source_payload(scenario, value)]
            if scenario.scenario_id == "c2-primary-source-exactly-once":
                sources.append({
                    "sequenceNumber": 2,
                    "reporterOrganization": "Second Reporter",
                    "reporterCountry": "KR",
                    "qualification": "1",
                    "primarySourceForRegulatoryPurposes": "1",
                })
            status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/RP", {
                "authorities": [scenario.authority],
                "rows": {"primarySources": sources},
            })
            read_status, current = page_current(case_id, "RP", scenario.owner)
            owner_id = object_id(current)
            if status != 200 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "rp_fixture_failed"})
                return
        elif scenario.page == "SD":
            status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/SD", {
                "authorities": [scenario.authority],
                "rows": {"senderInformation": sender_payload(scenario, value)},
            })
            read_status, current = page_current(case_id, "SD", scenario.owner)
            owner_id = object_id(current)
            if status != 200 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "sd_fixture_failed"})
                return
        elif scenario.page == "SI":
            status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/SI", {
                "authorities": [scenario.authority],
                "rows": {"studyInformation": study_payload(scenario, value)},
            })
            read_status, current = page_current(case_id, "SI", scenario.owner)
            owner_id = object_id(current)
            if status != 200 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "si_fixture_failed"})
                return
        elif scenario.page == "DM":
            rows: dict[str, Any] = {"patientInformation": {"patientInitials": "BUSINESS-FUZZ"}}
            if scenario.owner == "patientInformation":
                rows[scenario.owner] = patient_payload(scenario, value)
            elif scenario.owner == "medicalHistoryEpisodes":
                rows[scenario.owner] = [medical_history_payload(scenario, value)]
                if scenario.scenario_id == "d-history-parent-duplicate":
                    rows["parentInfo"] = {"parentSex": "2"}
                    rows["parentMedicalHistory"] = [{"meddraVersion": "26.0", "meddraCode": "10000001"}]
            elif scenario.owner == "deathInfo":
                rows[scenario.owner] = death_info_payload(scenario, value)
            elif scenario.owner in {"reportedCauses", "autopsyCauses"}:
                rows["deathInfo"] = {"dateOfDeath": f"{year}0303", "autopsyPerformed": True}
                rows[scenario.owner] = [death_cause_payload(scenario, value)]
            elif scenario.owner == "parentInfo":
                rows[scenario.owner] = parent_payload(scenario, value)
            elif scenario.owner == "parentMedicalHistory":
                rows["parentInfo"] = {"parentSex": "2"}
                rows[scenario.owner] = [parent_history_payload(scenario, value)]
            else:
                rows["parentInfo"] = {"parentSex": "2"}
                rows[scenario.owner] = [parent_past_drug_payload(scenario, value)]
            status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/DM", {
                "authorities": [scenario.authority],
                "rows": rows,
            })
            read_status, current = page_current(case_id, "DM", scenario.owner)
            owner_id = object_id(current)
            if status != 200 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "dm_fixture_failed"})
                return
        elif scenario.page == "NR":
            status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/NR", {
                "authorities": [scenario.authority],
                "rows": {
                    "narrative": {"caseNarrative": "Business fuzz narrative"},
                    scenario.owner: [narrative_payload(scenario, value)],
                },
            })
            read_status, current = page_current(case_id, "NR", scenario.owner)
            owner_id = object_id(current)
            if status != 200 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "nr_fixture_failed"})
                return
        elif scenario.page == "DH":
            status, _, _ = request("PATCH", f"/api/cases/{case_id}/editor/pages/DM", {
                "authorities": [scenario.authority],
                "rows": {"patientInformation": {"patientInitials": "BUSINESS-FUZZ"}},
            })
            if status != 200:
                add(edge, scenario, "FAIL", status, {"reason": "dh_patient_fixture_failed"})
                return
            status, created, save_summary = request("POST", f"/api/cases/{case_id}/editor/pages/DH/rows", {
                "authorities": [scenario.authority],
                "rows": {"pastDrugHistory": past_drug_payload(scenario, value)},
            })
            owner_id = created_row_id(created)
            if status != 201 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "dh_fixture_failed"})
                return
            read_status, current = page_current(case_id, "DH", scenario.owner, owner_id)
        elif scenario.page == "DG":
            payload = drug_payload(scenario, value)
            if scenario.field.startswith("drugReactionAssessments[]"):
                status, reaction, save_summary = request("POST", f"/api/cases/{case_id}/editor/pages/AE/rows", {
                    "authorities": [scenario.authority],
                    "rows": {"reaction": reaction_payload()},
                })
                reaction_id = created_row_id(reaction)
                if status != 201 or not reaction_id:
                    add(edge, scenario, "FAIL", status, {**save_summary, "reason": "dg_reaction_fixture_failed"})
                    return
                set_path(payload, "drugReactionAssessments[].reactionId", reaction_id)
            status, created, save_summary = request("POST", f"/api/cases/{case_id}/editor/pages/DG/rows", {
                "authorities": [scenario.authority],
                "rows": {"drug": payload},
            })
            owner_id = created_row_id(created)
            if status != 201 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "dg_fixture_failed"})
                return
            read_status, current = page_current(case_id, "DG", scenario.owner, owner_id)
        elif scenario.page == "AE":
            status, created, save_summary = request("POST", f"/api/cases/{case_id}/editor/pages/AE/rows", {
                "authorities": [scenario.authority],
                "rows": {"reaction": reaction_payload(scenario, value)},
            })
            owner_id = created_row_id(created)
            if status != 201 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "ae_fixture_failed"})
                return
            read_status, current = page_current(case_id, "AE", scenario.owner, owner_id)
        else:
            status, created, save_summary = request("POST", f"/api/cases/{case_id}/editor/pages/LB/rows", {
                "authorities": [scenario.authority],
                "rows": {"testResult": test_result_payload(scenario, value)},
            })
            owner_id = created_row_id(created)
            if status != 201 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "lb_fixture_failed"})
                return
            read_status, current = page_current(case_id, "LB", scenario.owner, owner_id)

        actual = get_path(current, scenario.projection_field)
        logs = audit_logs(case_id, scenario.owner, owner_id, scenario.field)
        complete_logs = [log for log in logs if audit_log_complete(log)]
        field_match = any(
            audit_key_matches(log.get("changedFields", log.get("changed_fields", {})), scenario.projection_field)
            for log in complete_logs
        )
        validation_status, report, validation_summary = validation(case_id, scenario.authority)
        present = scenario.expected_code in issue_codes(report)
        expected_present = edge == "invalid_edge"
        passed = (
            read_status == 200
            and values_equal(value, actual)
            and bool(complete_logs)
            and (value is None or field_match)
            and validation_status == 200
            and present == expected_present
            and (not expected_present or issue_complete(report, scenario.expected_code))
        )
        add(edge, scenario, "PASS" if passed else "FAIL", status, {
            **save_summary,
            "validation": validation_summary,
            "expected_code": scenario.expected_code,
            "expected_code_present": expected_present,
            "actual_code_present": present,
            "readback": redacted(actual),
            "audit_logs": len(logs),
            "audit_complete": bool(complete_logs),
            "audit_field_match": field_match,
        })

    ordered = scenarios[:]
    random.Random(args.seed).shuffle(ordered)
    for scenario in ordered:
        if interrupted:
            break
        run_edge(scenario, "invalid_edge", scenario.invalid_value)
        if not interrupted:
            run_edge(scenario, "valid_edge", scenario.valid_value)

    if not interrupted:
        status, value, summary = request("GET", "/api/audit-logs/verify-integrity")
        broken = value.get("broken_rows", value.get("brokenRows")) if isinstance(value, dict) else None
        add("audit_chain", None, "PASS" if status == 200 and broken == 0 else "FAIL", status, {
            **summary,
            "broken_rows": broken,
        })

    out_dir = Path(args.artifact_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    artifact = out_dir / f"case-business-validator-{args.seed}.jsonl"
    with artifact.open("w", encoding="utf-8") as handle:
        for event in events:
            handle.write(json.dumps({"seed": args.seed, "commit": commit_sha(), **asdict(event)}, sort_keys=True) + "\n")
        handle.write(json.dumps({
            "kind": "run",
            "seed": args.seed,
            "requests": requests,
            "elapsed_seconds": round(time.monotonic() - started, 3),
            "interrupted": interrupted,
            "scenario_count": len(scenarios),
            "covered_rules": sorted(covered),
            "inventory_rule_count": len(inventory),
            "uncovered_rules": sorted(inventory - covered),
            "artifact": str(artifact),
            "surface": "case-validation-api",
        }, sort_keys=True) + "\n")
    counts: dict[str, int] = {}
    for event in events:
        counts[event.classification] = counts.get(event.classification, 0) + 1
    print(f"events={len(events)} counts={json.dumps(counts, sort_keys=True)} artifact={artifact}")
    return 2 if interrupted else 1 if any(event.classification != "PASS" for event in events) else 0


if __name__ == "__main__":
    sys.exit(main(parser().parse_args()))
