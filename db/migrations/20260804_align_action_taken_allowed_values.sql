ALTER TABLE drug_information
DROP CONSTRAINT IF EXISTS drug_information_action_taken_check;

ALTER TABLE drug_information
ADD CONSTRAINT drug_information_action_taken_check
CHECK (action_taken IN ('0', '1', '2', '3', '4', '5', '6', '9'));
