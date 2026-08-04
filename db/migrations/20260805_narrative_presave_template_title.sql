ALTER TABLE narrative_presaves
    ADD COLUMN IF NOT EXISTS template_title VARCHAR(255);
