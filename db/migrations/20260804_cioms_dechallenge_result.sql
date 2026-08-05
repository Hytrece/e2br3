-- CIOMS Item 20: did the reaction abate after stopping the drug?
-- 1 = Yes, 2 = No, 3 = Not applicable. NULL means not entered yet.
ALTER TABLE drug_reaction_assessments
    ADD COLUMN IF NOT EXISTS dechallenge_result VARCHAR(1);

ALTER TABLE drug_reaction_assessments
    DROP CONSTRAINT IF EXISTS drug_reaction_assessments_dechallenge_result_check;

ALTER TABLE drug_reaction_assessments
    ADD CONSTRAINT drug_reaction_assessments_dechallenge_result_check
    CHECK (dechallenge_result IN ('1', '2', '3'));

BEGIN;
SELECT set_current_user_context('00000000-0000-0000-0000-000000000001');

INSERT INTO e2b_code_lists (list_name, code, display_name, sort_order)
VALUES
    ('dechallenge_result', '1', 'Yes', 1),
    ('dechallenge_result', '2', 'No', 2),
    ('dechallenge_result', '3', 'Not applicable', 3)
ON CONFLICT (list_name, code) DO NOTHING;
COMMIT;
