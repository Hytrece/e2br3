ALTER TABLE primary_sources
    DROP COLUMN IF EXISTS country_code_null_flavor;

ALTER TABLE reporter_presaves
    DROP COLUMN IF EXISTS country_code_null_flavor;
