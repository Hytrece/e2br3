-- Match persisted unit columns to the 50-character E2B input contract.
ALTER TABLE patient_information
    ALTER COLUMN age_unit TYPE VARCHAR(50),
    ALTER COLUMN gestation_period_unit TYPE VARCHAR(50);

ALTER TABLE parent_information
    ALTER COLUMN parent_age_unit TYPE VARCHAR(50);

ALTER TABLE reactions
    ALTER COLUMN duration_unit TYPE VARCHAR(50);

ALTER TABLE dosage_information
    ALTER COLUMN duration_unit TYPE VARCHAR(50);

ALTER TABLE drug_reaction_assessments
    ALTER COLUMN administration_start_interval_unit TYPE VARCHAR(50),
    ALTER COLUMN last_dose_interval_unit TYPE VARCHAR(50);
