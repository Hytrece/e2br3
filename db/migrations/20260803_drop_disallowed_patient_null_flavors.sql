DROP TRIGGER IF EXISTS trg_nfv_parent_information_62b167be8d ON parent_information;

ALTER TABLE patient_information
    DROP COLUMN IF EXISTS age_at_time_of_onset_null_flavor,
    DROP COLUMN IF EXISTS weight_kg_null_flavor,
    DROP COLUMN IF EXISTS height_cm_null_flavor;

ALTER TABLE parent_information
    DROP COLUMN IF EXISTS parent_age_null_flavor;

ALTER TABLE parent_past_drug_history
    DROP COLUMN IF EXISTS drug_name_null_flavor;

CREATE TRIGGER trg_nfv_parent_information_62b167be8d
BEFORE INSERT OR UPDATE ON parent_information
FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
    'last_menstrual_period_date', 'last_menstrual_period_date_null_flavor',
    'parent_birth_date', 'parent_birth_date_null_flavor',
    'parent_identification', 'parent_identification_null_flavor',
    'sex', 'sex_null_flavor'
);
