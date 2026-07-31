ALTER TABLE relatedness_assessments
    ADD COLUMN IF NOT EXISTS method_of_assessment_kr1 VARCHAR(10),
    ADD COLUMN IF NOT EXISTS result_of_assessment_kr1 VARCHAR(10);
