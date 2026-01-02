-- Seed data for E2B(R3) schema (dev only).
DO $$
DECLARE
    v_org_id BIGINT := 1000;
    v_user_id UUID := '11111111-1111-1111-1111-111111111111';
    v_case_id UUID := '22222222-2222-2222-2222-222222222222';
    v_case_version_id UUID := '22222222-2222-2222-2222-222222222223';
    v_message_header_id UUID := '33333333-3333-3333-3333-333333333333';
    v_safety_ident_id UUID := '44444444-4444-4444-4444-444444444444';
    v_sender_info_id UUID := '55555555-5555-5555-5555-555555555555';
    v_study_info_id UUID := '66666666-6666-6666-6666-666666666666';
    v_study_reg_id UUID := '66666666-6666-6666-6666-666666666667';
    v_primary_source_id UUID := '77777777-7777-7777-7777-777777777777';
    v_patient_id UUID := '88888888-8888-8888-8888-888888888888';
    v_med_history_id UUID := '88888888-8888-8888-8888-888888888889';
    v_past_drug_id UUID := '88888888-8888-8888-8888-888888888890';
    v_death_info_id UUID := '88888888-8888-8888-8888-888888888891';
    v_reported_death_id UUID := '88888888-8888-8888-8888-888888888892';
    v_autopsy_death_id UUID := '88888888-8888-8888-8888-888888888893';
    v_parent_info_id UUID := '88888888-8888-8888-8888-888888888894';
    v_reaction_id UUID := '99999999-9999-9999-9999-999999999999';
    v_test_result_id UUID := '99999999-9999-9999-9999-999999999998';
    v_drug_info_id UUID := 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
    v_drug_substance_id UUID := 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaab';
    v_dosage_info_id UUID := 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaac';
    v_drug_indication_id UUID := 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaad';
    v_narrative_id UUID := 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb';
    v_sender_diag_id UUID := 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbc';
    v_case_summary_id UUID := 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbd';
    v_literature_ref_id UUID := 'cccccccc-cccc-cccc-cccc-cccccccccccc';
BEGIN
    INSERT INTO organizations (id, name, type, address, city, state, postcode, country_code, contact_email, contact_phone, active, cid, ctime, mid, mtime)
    VALUES (v_org_id, 'Demo Organization', 'internal', '123 Demo St', 'Metropolis', 'CA', '12345', 'US', 'demo@example.com', '555-1234', true, 0, NOW(), 0, NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO users (id, organization_id, email, username, role, active, cid, ctime, mid, mtime)
    VALUES (v_user_id, v_org_id, 'demo.user@example.com', 'demo_user', 'admin', true, 0, NOW(), 0, NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO cases (id, organization_id, safety_report_id, version, status, created_by, updated_by, submitted_by, submitted_at, created_at, updated_at)
    VALUES (v_case_id, v_org_id, 'SR-001', 1, 'draft', v_user_id, v_user_id, v_user_id, NOW(), NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO case_versions (id, case_id, version, snapshot, changed_by, change_reason, created_at)
    VALUES (v_case_version_id, v_case_id, 1, '{}'::JSONB, v_user_id, 'Initial import', NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO message_headers (id, case_id, batch_number, batch_sender_identifier, message_type, message_format_version, message_format_release, message_number, message_sender_identifier, message_receiver_identifier, message_date_format, message_date, created_at, updated_at)
    VALUES (v_message_header_id, v_case_id, 'B-001', 'SENDER-1', 'ichicsr', '2.1', '2.0', 'MSG-001', 'ORG-SENDER', 'ORG-RECEIVER', '204', '20240101120000', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO safety_report_identification (id, case_id, transmission_date, report_type, date_first_received_from_source, date_of_most_recent_information, fulfil_expedited_criteria, receiver_organization, created_at, updated_at)
    VALUES (v_safety_ident_id, v_case_id, CURRENT_DATE, '1', CURRENT_DATE, CURRENT_DATE, TRUE, 'Demo Receiver', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO sender_information (id, case_id, sender_type, organization_name, created_at, updated_at)
    VALUES (v_sender_info_id, v_case_id, '1', 'Demo Sender Org', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO literature_references (id, case_id, reference_text, sequence_number, created_at)
    VALUES (v_literature_ref_id, v_case_id, 'Sample literature reference', 1, NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO study_information (id, case_id, study_name, sponsor_study_number, study_type_reaction, created_at, updated_at)
    VALUES (v_study_info_id, v_case_id, 'Study A', 'SSN-1', '01', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO study_registration_numbers (id, study_information_id, registration_number, country_code, sequence_number)
    VALUES (v_study_reg_id, v_study_info_id, 'REG-001', 'US', 1)
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO primary_sources (id, case_id, sequence_number, reporter_given_name, reporter_family_name, country_code, qualification, primary_source_regulatory, created_at, updated_at)
    VALUES (v_primary_source_id, v_case_id, 1, 'Jane', 'Doe', 'US', '1', '1', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO patient_information (id, case_id, patient_initials, sex, created_at, updated_at)
    VALUES (v_patient_id, v_case_id, 'JD', '1', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO medical_history_episodes (id, patient_id, sequence_number, meddra_version, meddra_code, start_date, created_at)
    VALUES (v_med_history_id, v_patient_id, 1, '26.0', '12345678', CURRENT_DATE - 365, NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO past_drug_history (id, patient_id, sequence_number, drug_name, start_date, end_date, created_at)
    VALUES (v_past_drug_id, v_patient_id, 1, 'Historical Drug', CURRENT_DATE - 400, CURRENT_DATE - 350, NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO patient_death_information (id, patient_id, date_of_death, autopsy_performed, created_at, updated_at)
    VALUES (v_death_info_id, v_patient_id, CURRENT_DATE - 1, FALSE, NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO reported_causes_of_death (id, death_info_id, sequence_number, meddra_version, meddra_code)
    VALUES (v_reported_death_id, v_death_info_id, 1, '26.0', '87654321')
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO autopsy_causes_of_death (id, death_info_id, sequence_number, meddra_version, meddra_code)
    VALUES (v_autopsy_death_id, v_death_info_id, 1, '26.0', '87654322')
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO parent_information (id, patient_id, parent_identification, sex, created_at, updated_at)
    VALUES (v_parent_info_id, v_patient_id, 'Parent-1', '2', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO reactions (id, case_id, sequence_number, primary_source_reaction, serious, outcome, created_at, updated_at)
    VALUES (v_reaction_id, v_case_id, 1, 'Headache', FALSE, '0', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO test_results (id, case_id, sequence_number, test_date, test_name, created_at, updated_at)
    VALUES (v_test_result_id, v_case_id, 1, CURRENT_DATE, 'Blood Test', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO drug_information (id, case_id, sequence_number, drug_characterization, medicinal_product, action_taken, created_at, updated_at)
    VALUES (v_drug_info_id, v_case_id, 1, '1', 'Demo Drug', '1', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO drug_active_substances (id, drug_id, sequence_number, substance_name, strength_value, strength_unit)
    VALUES (v_drug_substance_id, v_drug_info_id, 1, 'Substance A', 10.0, 'mg')
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO dosage_information (id, drug_id, sequence_number, dose_value, dose_unit, frequency_value, frequency_unit, created_at, updated_at)
    VALUES (v_dosage_info_id, v_drug_info_id, 1, 1.0, 'tab', 1.0, 'day', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO drug_indications (id, drug_id, sequence_number, indication_text, indication_meddra_version, indication_meddra_code)
    VALUES (v_drug_indication_id, v_drug_info_id, 1, 'Indication text', '26.0', '135790')
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO narrative_information (id, case_id, case_narrative, reporter_comments, sender_comments, created_at, updated_at)
    VALUES (v_narrative_id, v_case_id, 'Case narrative text', 'Reporter comment', 'Sender comment', NOW(), NOW())
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO sender_diagnoses (id, narrative_id, sequence_number, diagnosis_meddra_version, diagnosis_meddra_code)
    VALUES (v_sender_diag_id, v_narrative_id, 1, '26.0', '246810')
    ON CONFLICT (id) DO NOTHING;

    INSERT INTO case_summary_information (id, narrative_id, sequence_number, summary_type, language_code, summary_text)
    VALUES (v_case_summary_id, v_narrative_id, 1, '01', 'en', 'Case summary text')
    ON CONFLICT (id) DO NOTHING;
END;
$$;
