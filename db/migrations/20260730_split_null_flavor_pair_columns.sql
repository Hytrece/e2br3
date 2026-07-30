ALTER TABLE dosage_information
    ADD COLUMN IF NOT EXISTS dose_form_null_flavor VARCHAR(4),
    ADD COLUMN IF NOT EXISTS route_of_administration_null_flavor VARCHAR(4),
    ADD COLUMN IF NOT EXISTS parent_route_null_flavor VARCHAR(4),
    DROP CONSTRAINT IF EXISTS ck_nfv_dosage_information_f3e628bb02f1,
    ADD CONSTRAINT ck_nfv_dosage_information_f3e628bb02f1
        CHECK (dose_form IS NULL OR dose_form_null_flavor IS NULL),
    DROP CONSTRAINT IF EXISTS ck_nfv_dosage_information_12f028792e1f,
    ADD CONSTRAINT ck_nfv_dosage_information_12f028792e1f
        CHECK (route_of_administration IS NULL OR route_of_administration_null_flavor IS NULL),
    DROP CONSTRAINT IF EXISTS ck_nfv_dosage_information_f27137b32806,
    ADD CONSTRAINT ck_nfv_dosage_information_f27137b32806
        CHECK (parent_route IS NULL OR parent_route_null_flavor IS NULL);

ALTER TABLE test_results
    ADD COLUMN IF NOT EXISTS test_result_null_flavor VARCHAR(4),
    DROP CONSTRAINT IF EXISTS ck_nfv_test_results_3f503f260cf6,
    ADD CONSTRAINT ck_nfv_test_results_3f503f260cf6
        CHECK (test_result_value IS NULL OR test_result_null_flavor IS NULL);

DROP TRIGGER IF EXISTS trg_nfv_dosage_information_52a6bc6d6b
    ON dosage_information;
CREATE TRIGGER trg_nfv_dosage_information_52a6bc6d6b
    BEFORE INSERT OR UPDATE ON dosage_information
    FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
        'batch_lot_number', 'batch_lot_number_null_flavor',
        'dose_form', 'dose_form_null_flavor',
        'first_administration_date', 'first_administration_date_null_flavor',
        'last_administration_date', 'last_administration_date_null_flavor',
        'parent_route', 'parent_route_null_flavor',
        'route_of_administration', 'route_of_administration_null_flavor'
    );

DROP TRIGGER IF EXISTS trg_nfv_test_results_0b13e32562
    ON test_results;
CREATE TRIGGER trg_nfv_test_results_0b13e32562
    BEFORE INSERT OR UPDATE ON test_results
    FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
        'test_date', 'test_date_null_flavor',
        'test_result_value', 'test_result_null_flavor'
    );
