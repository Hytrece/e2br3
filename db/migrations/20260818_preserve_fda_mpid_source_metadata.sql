ALTER TABLE drug_information
    ADD COLUMN IF NOT EXISTS mpid_source_code_system TEXT,
    ADD COLUMN IF NOT EXISTS mpid_source_code_system_version TEXT;

ALTER TABLE past_drug_history
    ADD COLUMN IF NOT EXISTS mpid_source_code_system TEXT,
    ADD COLUMN IF NOT EXISTS mpid_source_code_system_version TEXT;
