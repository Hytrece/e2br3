ALTER TABLE drug_reaction_assessments
    ADD COLUMN IF NOT EXISTS expectedness VARCHAR(1);

ALTER TABLE drug_reaction_assessments
    DROP CONSTRAINT IF EXISTS drug_reaction_assessments_expectedness_check,
    ADD CONSTRAINT drug_reaction_assessments_expectedness_check
        CHECK (expectedness IS NULL OR expectedness IN ('1', '2'));
