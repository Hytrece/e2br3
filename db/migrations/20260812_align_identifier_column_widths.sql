ALTER TABLE past_drug_history
    ALTER COLUMN mpid TYPE VARCHAR(1000),
    ALTER COLUMN phpid TYPE VARCHAR(250);

ALTER TABLE parent_past_drug_history
    ALTER COLUMN mpid TYPE VARCHAR(1000),
    ALTER COLUMN phpid TYPE VARCHAR(250);

ALTER TABLE drug_information
    ALTER COLUMN mpid TYPE VARCHAR(1000),
    ALTER COLUMN phpid TYPE VARCHAR(250);

ALTER TABLE dosage_information
    ALTER COLUMN dose_form_termid TYPE VARCHAR(100),
    ALTER COLUMN route_termid TYPE VARCHAR(100),
    ALTER COLUMN parent_route TYPE VARCHAR(60),
    ALTER COLUMN parent_route_termid TYPE VARCHAR(100);

ALTER TABLE relatedness_assessments
    ALTER COLUMN result_of_assessment TYPE VARCHAR(60);

ALTER TABLE other_case_identifiers
    ALTER COLUMN source_of_identifier TYPE VARCHAR(100);
