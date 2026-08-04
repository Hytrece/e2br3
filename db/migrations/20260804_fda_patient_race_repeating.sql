ALTER TABLE patient_information
    ADD COLUMN IF NOT EXISTS race_codes VARCHAR(10)[] NOT NULL DEFAULT '{}';

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = current_schema()
          AND table_name = 'patient_information'
          AND column_name = 'race_code'
    ) THEN
        EXECUTE 'UPDATE patient_information
                 SET race_codes = ARRAY[race_code]::VARCHAR(10)[]
                 WHERE race_code IS NOT NULL';
    END IF;
END
$$;

ALTER TABLE patient_information
    DROP CONSTRAINT IF EXISTS ck_nfv_patient_informatio_d80509c0f626,
    DROP CONSTRAINT IF EXISTS ck_patient_race_codes,
    DROP CONSTRAINT IF EXISTS ck_patient_race_null_flavor_pair,
    DROP COLUMN IF EXISTS race_code,
    ADD CONSTRAINT ck_patient_race_codes CHECK (
        race_codes <@ ARRAY['C16352', 'C41259', 'C41260', 'C41219', 'C41261']::VARCHAR(10)[]
    ),
    ADD CONSTRAINT ck_patient_race_null_flavor_pair CHECK (
        cardinality(race_codes) = 0 OR race_code_null_flavor IS NULL
    );

-- The generic NullFlavor trigger was created with race_code as an argument.
-- Rebuild it from the pairs that still use scalar columns.
DROP TRIGGER IF EXISTS trg_nfv_patient_information_705c7159d6 ON patient_information;
CREATE TRIGGER trg_nfv_patient_information_705c7159d6
BEFORE INSERT OR UPDATE ON patient_information
FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
    'birth_date', 'birth_date_null_flavor',
    'ethnicity_code', 'ethnicity_code_null_flavor',
    'last_menstrual_period_date', 'last_menstrual_period_date_null_flavor',
    'medical_history_text', 'medical_history_text_null_flavor',
    'patient_initials', 'patient_initials_null_flavor',
    'sex', 'sex_null_flavor'
);
