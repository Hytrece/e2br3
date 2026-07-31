ALTER TABLE relatedness_assessments
    ADD COLUMN IF NOT EXISTS result_of_assessment_kr1_null_flavor VARCHAR(10);

UPDATE relatedness_assessments
SET result_of_assessment_kr1 = NULL,
    result_of_assessment_kr1_null_flavor = 'NA'
WHERE result_of_assessment_kr1 = 'NA'
  AND result_of_assessment_kr1_null_flavor IS NULL;
