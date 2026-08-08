ALTER TABLE cases
    ADD COLUMN IF NOT EXISTS import_authority VARCHAR(16);

ALTER TABLE cases
    DROP CONSTRAINT IF EXISTS cases_import_authority_valid;

ALTER TABLE cases
    ADD CONSTRAINT cases_import_authority_valid
    CHECK (import_authority IS NULL OR import_authority IN ('ich', 'fda', 'mfds'));
