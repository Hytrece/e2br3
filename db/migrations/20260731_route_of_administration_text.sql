-- e2b:G.k.4.r.10.1 is route text; G.k.4.r.10.2b stores the terminology code.
ALTER TABLE dosage_information
    ALTER COLUMN route_of_administration TYPE VARCHAR(200);
