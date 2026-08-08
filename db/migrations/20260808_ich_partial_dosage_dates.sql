ALTER TABLE dosage_information
    ADD COLUMN IF NOT EXISTS first_administration_date_raw TEXT,
    ADD COLUMN IF NOT EXISTS last_administration_date_raw TEXT;
