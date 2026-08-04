-- Standalone legacy NINF/PINF rows have no numeric bound. Drop them rather
-- than inventing a bound during conversion.
ALTER TABLE test_results
    DROP COLUMN IF EXISTS test_result_null_flavor,
    ADD COLUMN IF NOT EXISTS test_result_qualifier VARCHAR(2),
    DROP CONSTRAINT IF EXISTS ck_test_result_qualifier,
    ADD CONSTRAINT ck_test_result_qualifier
        CHECK (test_result_qualifier IS NULL OR test_result_qualifier IN ('EQ', 'LT', 'LE', 'GT', 'GE')),
    DROP CONSTRAINT IF EXISTS ck_test_result_qualifier_value,
    ADD CONSTRAINT ck_test_result_qualifier_value
        CHECK (test_result_qualifier IS NULL OR test_result_value IS NOT NULL);

DROP TRIGGER IF EXISTS trg_nfv_test_results_0b13e32562 ON test_results;
CREATE TRIGGER trg_nfv_test_results_0b13e32562
    BEFORE INSERT OR UPDATE ON test_results
    FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
        'test_date', 'test_date_null_flavor'
    );
