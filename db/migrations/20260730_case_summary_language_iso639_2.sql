ALTER TABLE case_summary_information
    ALTER COLUMN language_code TYPE VARCHAR(3);

ALTER TABLE case_summary_information
    DROP COLUMN IF EXISTS summary_type;
