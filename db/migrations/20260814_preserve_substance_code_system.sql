ALTER TABLE drug_active_substances
	ADD COLUMN IF NOT EXISTS substance_termid_code_system TEXT;
