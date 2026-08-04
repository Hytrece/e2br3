ALTER TABLE literature_references
    ADD COLUMN IF NOT EXISTS file_name TEXT;

ALTER TABLE documents_held_by_sender
    ADD COLUMN IF NOT EXISTS file_name TEXT;
