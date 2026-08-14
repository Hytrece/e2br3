ALTER TABLE dosage_information
	-- Source XML round-trip storage; G.k.4.r.7 input contracts remain unchanged.
	ADD COLUMN IF NOT EXISTS batch_lot_number_null_flavor VARCHAR(4),
	DROP CONSTRAINT IF EXISTS dosage_information_batch_lot_number_null_flavor_check,
	ADD CONSTRAINT dosage_information_batch_lot_number_null_flavor_check
		CHECK (batch_lot_number_null_flavor IS NULL OR batch_lot_number_null_flavor = 'UNK'),
	DROP CONSTRAINT IF EXISTS ck_nfv_dosage_information_batch_lot,
	ADD CONSTRAINT ck_nfv_dosage_information_batch_lot
		CHECK (batch_lot_number IS NULL OR batch_lot_number_null_flavor IS NULL);
