ALTER TABLE sender_information
	ADD COLUMN IF NOT EXISTS person_title_null_flavor VARCHAR(4) CHECK (person_title_null_flavor IN ('MSK', 'ASKU', 'NASK')),
	ADD COLUMN IF NOT EXISTS person_given_name_null_flavor VARCHAR(4) CHECK (person_given_name_null_flavor IN ('MSK', 'ASKU', 'NASK')),
	ADD COLUMN IF NOT EXISTS person_middle_name_null_flavor VARCHAR(4) CHECK (person_middle_name_null_flavor IN ('MSK', 'ASKU', 'NASK')),
	ADD COLUMN IF NOT EXISTS person_family_name_null_flavor VARCHAR(4) CHECK (person_family_name_null_flavor IN ('MSK', 'ASKU', 'NASK'));
