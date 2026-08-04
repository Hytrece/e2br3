ALTER TABLE patient_information
    DROP CONSTRAINT IF EXISTS ck_nf_patient_initials,
    DROP CONSTRAINT IF EXISTS patient_information_patient_initials_null_flavor_check,
    ADD CONSTRAINT ck_nf_patient_initials
        CHECK (patient_initials_null_flavor IS NULL OR patient_initials_null_flavor IN ('MSK', 'UNK', 'ASKU', 'NASK', 'NA'));

ALTER TABLE drug_information
    ADD COLUMN IF NOT EXISTS fda_additional_info_coded_null_flavor VARCHAR(2),
    DROP CONSTRAINT IF EXISTS drug_information_fda_additional_info_coded_null_flavor_check,
    DROP CONSTRAINT IF EXISTS ck_nf_fda_drug_additional_info,
    DROP CONSTRAINT IF EXISTS ck_pair_fda_drug_additional_info,
    ADD CONSTRAINT ck_nf_fda_drug_additional_info
        CHECK (fda_additional_info_coded_null_flavor IS NULL OR fda_additional_info_coded_null_flavor = 'NA'),
    ADD CONSTRAINT ck_pair_fda_drug_additional_info
        CHECK (NOT (fda_additional_info_coded IS NOT NULL AND fda_additional_info_coded_null_flavor IS NOT NULL));
