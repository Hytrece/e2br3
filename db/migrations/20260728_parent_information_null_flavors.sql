ALTER TABLE parent_information
    ADD COLUMN IF NOT EXISTS parent_identification_null_flavor VARCHAR(4),
    ADD COLUMN IF NOT EXISTS sex_null_flavor VARCHAR(4);

ALTER TABLE parent_information
    DROP CONSTRAINT IF EXISTS ck_parent_identification_null_flavor_allowed,
    ADD CONSTRAINT ck_parent_identification_null_flavor_allowed
        CHECK (parent_identification_null_flavor IS NULL OR parent_identification_null_flavor IN ('MSK', 'UNK', 'ASKU', 'NASK')),
    DROP CONSTRAINT IF EXISTS ck_parent_sex_null_flavor_allowed,
    ADD CONSTRAINT ck_parent_sex_null_flavor_allowed
        CHECK (sex_null_flavor IS NULL OR sex_null_flavor IN ('MSK', 'UNK', 'ASKU', 'NASK')),
    DROP CONSTRAINT IF EXISTS ck_nfv_parent_information_identification,
    ADD CONSTRAINT ck_nfv_parent_information_identification
        CHECK (parent_identification IS NULL OR parent_identification_null_flavor IS NULL),
    DROP CONSTRAINT IF EXISTS ck_nfv_parent_information_sex,
    ADD CONSTRAINT ck_nfv_parent_information_sex
        CHECK (sex IS NULL OR sex_null_flavor IS NULL);

DROP TRIGGER IF EXISTS trg_nfv_parent_information_62b167be8d ON parent_information;
CREATE TRIGGER trg_nfv_parent_information_62b167be8d
BEFORE INSERT OR UPDATE ON parent_information
FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
    'last_menstrual_period_date', 'last_menstrual_period_date_null_flavor',
    'parent_age', 'parent_age_null_flavor',
    'parent_birth_date', 'parent_birth_date_null_flavor',
    'parent_identification', 'parent_identification_null_flavor',
    'sex', 'sex_null_flavor'
);
