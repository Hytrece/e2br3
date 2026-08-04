ALTER TABLE reporter_presaves
    ADD COLUMN IF NOT EXISTS reporter_email VARCHAR(100);
