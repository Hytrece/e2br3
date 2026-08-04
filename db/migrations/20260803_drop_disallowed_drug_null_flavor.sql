DROP TRIGGER IF EXISTS trg_nfv_dosage_information_52a6bc6d6b
    ON dosage_information;

ALTER TABLE dosage_information
    DROP COLUMN IF EXISTS batch_lot_number_null_flavor;

CREATE TRIGGER trg_nfv_dosage_information_52a6bc6d6b
    BEFORE INSERT OR UPDATE ON dosage_information
    FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(
        'dose_form', 'dose_form_null_flavor',
        'first_administration_date', 'first_administration_date_null_flavor',
        'last_administration_date', 'last_administration_date_null_flavor',
        'parent_route', 'parent_route_null_flavor',
        'route_of_administration', 'route_of_administration_null_flavor'
    );
