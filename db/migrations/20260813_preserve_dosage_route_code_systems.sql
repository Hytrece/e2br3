ALTER TABLE dosage_information
	ADD COLUMN IF NOT EXISTS route_termid_code_system TEXT,
	ADD COLUMN IF NOT EXISTS parent_route_termid_code_system TEXT;
