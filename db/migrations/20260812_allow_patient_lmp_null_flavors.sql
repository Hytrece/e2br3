ALTER TABLE patient_information
    DROP CONSTRAINT IF EXISTS patient_information_last_menstrual_period_date_null_flavo_check,
    DROP CONSTRAINT IF EXISTS patient_information_last_menstrual_period_date_null_flavor_check,
    DROP CONSTRAINT IF EXISTS ck_nf_patient_lmp,
    ADD CONSTRAINT ck_nf_patient_lmp CHECK (
        last_menstrual_period_date_null_flavor IS NULL
        OR last_menstrual_period_date_null_flavor IN ('MSK', 'UNK', 'ASKU', 'NASK')
    );
