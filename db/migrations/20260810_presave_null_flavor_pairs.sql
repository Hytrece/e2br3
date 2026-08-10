ALTER TABLE reporter_presaves
    ADD COLUMN IF NOT EXISTS reporter_email_null_flavor VARCHAR(4)
        CHECK (reporter_email_null_flavor IN ('MSK', 'ASKU', 'NASK'));

ALTER TABLE study_presaves
    ADD COLUMN IF NOT EXISTS study_name_null_flavor VARCHAR(4)
        CHECK (study_name_null_flavor IN ('ASKU', 'NASK')),
    ADD COLUMN IF NOT EXISTS sponsor_study_number_null_flavor VARCHAR(4)
        CHECK (sponsor_study_number_null_flavor IN ('ASKU', 'NASK'));

ALTER TABLE study_presave_registration_numbers
    ADD COLUMN IF NOT EXISTS registration_number_null_flavor VARCHAR(4)
        CHECK (registration_number_null_flavor IN ('ASKU', 'NASK')),
    ADD COLUMN IF NOT EXISTS country_code_null_flavor VARCHAR(4)
        CHECK (country_code_null_flavor IN ('ASKU', 'NASK'));

ALTER TABLE study_presave_fda_cross_reported_ind_numbers
    ALTER COLUMN ind_number DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS ind_number_null_flavor VARCHAR(4)
        CHECK (ind_number_null_flavor IN ('NA'));

DO $$
DECLARE
    pair record;
    table_pairs record;
    constraint_name text;
    trigger_name text;
BEGIN
    FOR pair IN
        SELECT nf.table_name,
               nf.column_name AS null_flavor_column,
               replace(nf.column_name, '_null_flavor', '') AS value_column
        FROM information_schema.columns nf
        WHERE nf.table_schema = 'public'
          AND nf.column_name LIKE '%\_null\_flavor' ESCAPE '\'
          AND EXISTS (
              SELECT 1 FROM information_schema.columns value
              WHERE value.table_schema = nf.table_schema
                AND value.table_name = nf.table_name
                AND value.column_name = replace(nf.column_name, '_null_flavor', '')
          )
        ORDER BY nf.table_name, nf.column_name
    LOOP
        constraint_name := format(
            'ck_nfv_%s_%s',
            substr(pair.table_name, 1, 20),
            substr(md5(pair.table_name || ':' || pair.value_column), 1, 12)
        );
        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = constraint_name) THEN
            EXECUTE format(
                'ALTER TABLE %I ADD CONSTRAINT %I CHECK (%I IS NULL OR %I IS NULL)',
                pair.table_name, constraint_name,
                pair.value_column, pair.null_flavor_column
            );
        END IF;
    END LOOP;

    FOR table_pairs IN
        SELECT nf.table_name,
               string_agg(
                   format('%L, %L', replace(nf.column_name, '_null_flavor', ''), nf.column_name),
                   ', ' ORDER BY nf.column_name
               ) AS trigger_arguments
        FROM information_schema.columns nf
        WHERE nf.table_schema = 'public'
          AND nf.column_name LIKE '%\_null\_flavor' ESCAPE '\'
          AND EXISTS (
              SELECT 1 FROM information_schema.columns value
              WHERE value.table_schema = nf.table_schema
                AND value.table_name = nf.table_name
                AND value.column_name = replace(nf.column_name, '_null_flavor', '')
          )
        GROUP BY nf.table_name
    LOOP
        trigger_name := format(
            'trg_nfv_%s_%s',
            substr(table_pairs.table_name, 1, 24),
            substr(md5(table_pairs.table_name), 1, 10)
        );
        EXECUTE format('DROP TRIGGER IF EXISTS %I ON %I', trigger_name, table_pairs.table_name);
        EXECUTE format(
            'CREATE TRIGGER %I BEFORE INSERT OR UPDATE ON %I
             FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(%s)',
            trigger_name, table_pairs.table_name, table_pairs.trigger_arguments
        );
    END LOOP;
END
$$;
