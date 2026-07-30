ALTER TABLE parent_medical_history
    ADD COLUMN IF NOT EXISTS continuing_null_flavor VARCHAR(4);

ALTER TABLE parent_medical_history
    DROP CONSTRAINT IF EXISTS ck_parent_medical_history_continuing_null_flavor_allowed,
    ADD CONSTRAINT ck_parent_medical_history_continuing_null_flavor_allowed
        CHECK (continuing_null_flavor IS NULL OR continuing_null_flavor IN ('MSK', 'UNK', 'ASKU', 'NASK')),
    DROP CONSTRAINT IF EXISTS ck_parent_medical_history_continuing_null_flavor_pair,
    ADD CONSTRAINT ck_parent_medical_history_continuing_null_flavor_pair
        CHECK (continuing IS NULL OR continuing_null_flavor IS NULL);

DROP TRIGGER IF EXISTS trg_parent_medical_history_continuing_nfv
    ON parent_medical_history;
CREATE TRIGGER trg_parent_medical_history_continuing_nfv
    BEFORE INSERT OR UPDATE ON parent_medical_history
    FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
        'continuing', 'continuing_null_flavor'
    );

ALTER TABLE reactions
    ALTER COLUMN required_intervention TYPE BOOLEAN
    USING CASE lower(required_intervention::text)
        WHEN 'true' THEN true
        WHEN '1' THEN true
        WHEN 'false' THEN false
        WHEN '0' THEN false
        WHEN '2' THEN false
        ELSE NULL
    END;
