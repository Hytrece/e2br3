-- Enforce the E2B invariant that a concrete value and its NullFlavor cannot
-- coexist. This is schema-driven so newly added *_null_flavor pairs are
-- protected without maintaining a second hand-written list.

-- Legacy versions of the case-editor PATCH path could preserve an old
-- NullFlavor while setting a concrete value. Preserve the usable concrete
-- value and clear the conflicting NullFlavor before installing constraints.
DO $$
DECLARE
    pair record;
BEGIN
    PERFORM set_config(
        'app.current_user_id',
        '00000000-0000-0000-0000-000000000001',
        true
    );
    PERFORM set_config(
        'app.current_organization_id',
        '00000000-0000-0000-0000-000000000001',
        true
    );
    PERFORM set_config('app.current_user_role', 'system_admin', true);

    FOR pair IN
        SELECT
            nf.table_name,
            nf.column_name AS null_flavor_column,
            replace(nf.column_name, '_null_flavor', '') AS value_column
        FROM information_schema.columns nf
        WHERE nf.table_schema = 'public'
          AND nf.column_name LIKE '%\_null\_flavor' ESCAPE '\'
          AND EXISTS (
              SELECT 1
              FROM information_schema.columns value
              WHERE value.table_schema = nf.table_schema
                AND value.table_name = nf.table_name
                AND value.column_name = replace(
                    nf.column_name,
                    '_null_flavor',
                    ''
                )
          )
    LOOP
        EXECUTE format(
            'UPDATE %I SET %I = NULL WHERE %I IS NOT NULL AND %I IS NOT NULL',
            pair.table_name,
            pair.null_flavor_column,
            pair.value_column,
            pair.null_flavor_column
        );
    END LOOP;
END
$$;

CREATE OR REPLACE FUNCTION enforce_value_null_flavor_exclusivity()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    row_data jsonb := to_jsonb(NEW);
    old_data jsonb := CASE WHEN TG_OP = 'UPDATE' THEN to_jsonb(OLD) ELSE '{}'::jsonb END;
    pair_index integer;
    value_column text;
    null_flavor_column text;
    value_present boolean;
    null_flavor_present boolean;
    value_changed boolean;
    null_flavor_changed boolean;
BEGIN
    IF TG_NARGS = 0 OR TG_NARGS % 2 <> 0 THEN
        RAISE EXCEPTION 'NullFlavor trigger on % has invalid pair arguments', TG_TABLE_NAME;
    END IF;

    FOR pair_index IN 0..(TG_NARGS / 2 - 1)
    LOOP
        value_column := TG_ARGV[pair_index * 2];
        null_flavor_column := TG_ARGV[pair_index * 2 + 1];
        value_present := row_data -> value_column IS DISTINCT FROM 'null'::jsonb;
        null_flavor_present := row_data -> null_flavor_column IS DISTINCT FROM 'null'::jsonb;

        IF value_present AND null_flavor_present THEN
            value_changed := TG_OP = 'INSERT'
                OR row_data -> value_column IS DISTINCT FROM old_data -> value_column;
            null_flavor_changed := TG_OP = 'INSERT'
                OR row_data -> null_flavor_column IS DISTINCT FROM old_data -> null_flavor_column;

            IF TG_OP = 'UPDATE' AND value_changed AND NOT null_flavor_changed THEN
                row_data := jsonb_set(
                    row_data,
                    ARRAY[null_flavor_column],
                    'null'::jsonb,
                    true
                );
            ELSIF TG_OP = 'UPDATE' AND null_flavor_changed AND NOT value_changed THEN
                row_data := jsonb_set(
                    row_data,
                    ARRAY[value_column],
                    'null'::jsonb,
                    true
                );
            ELSE
                RAISE EXCEPTION USING
                    ERRCODE = '23514',
                    MESSAGE = format(
                        '%s.%s and %s cannot both be non-null',
                        TG_TABLE_NAME,
                        value_column,
                        null_flavor_column
                    );
            END IF;
        END IF;
    END LOOP;

    NEW := jsonb_populate_record(NEW, row_data);
    RETURN NEW;
END
$$;

DO $$
DECLARE
    pair record;
    table_pairs record;
    constraint_name text;
    trigger_name text;
BEGIN
    FOR pair IN
        SELECT
            nf.table_name,
            nf.column_name AS null_flavor_column,
            replace(nf.column_name, '_null_flavor', '') AS value_column
        FROM information_schema.columns nf
        WHERE nf.table_schema = 'public'
          AND nf.column_name LIKE '%\_null\_flavor' ESCAPE '\'
          AND EXISTS (
              SELECT 1
              FROM information_schema.columns value
              WHERE value.table_schema = nf.table_schema
                AND value.table_name = nf.table_name
                AND value.column_name = replace(
                    nf.column_name,
                    '_null_flavor',
                    ''
                )
          )
        ORDER BY nf.table_name, nf.column_name
    LOOP
        constraint_name := format(
            'ck_nfv_%s_%s',
            substr(pair.table_name, 1, 20),
            substr(md5(pair.table_name || ':' || pair.value_column), 1, 12)
        );
        IF NOT EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE connamespace = 'public'::regnamespace
              AND conrelid = format('public.%I', pair.table_name)::regclass
              AND conname = constraint_name
        ) THEN
            EXECUTE format(
                'ALTER TABLE %I ADD CONSTRAINT %I CHECK (%I IS NULL OR %I IS NULL)',
                pair.table_name,
                constraint_name,
                pair.value_column,
                pair.null_flavor_column
            );
        END IF;
    END LOOP;

    FOR table_pairs IN
        SELECT
            nf.table_name,
            string_agg(
                format(
                    '%L, %L',
                    replace(nf.column_name, '_null_flavor', ''),
                    nf.column_name
                ),
                ', ' ORDER BY nf.column_name
            ) AS trigger_arguments
        FROM information_schema.columns nf
        WHERE nf.table_schema = 'public'
          AND nf.column_name LIKE '%\_null\_flavor' ESCAPE '\'
          AND EXISTS (
              SELECT 1
              FROM information_schema.columns value
              WHERE value.table_schema = nf.table_schema
                AND value.table_name = nf.table_name
                AND value.column_name = replace(
                    nf.column_name,
                    '_null_flavor',
                    ''
                )
          )
        GROUP BY nf.table_name
        ORDER BY nf.table_name
    LOOP
        trigger_name := format(
            'trg_nfv_%s_%s',
            substr(table_pairs.table_name, 1, 24),
            substr(md5(table_pairs.table_name), 1, 10)
        );
        EXECUTE format(
            'DROP TRIGGER IF EXISTS %I ON %I',
            trigger_name,
            table_pairs.table_name
        );
        EXECUTE format(
            'CREATE TRIGGER %I BEFORE INSERT OR UPDATE ON %I
             FOR EACH ROW EXECUTE FUNCTION enforce_value_null_flavor_exclusivity(%s)',
            trigger_name,
            table_pairs.table_name,
            table_pairs.trigger_arguments
        );
    END LOOP;
END
$$;
