-- Import duplicate decisions are retained in history instead of disappearing.
ALTER TABLE xml_import_history
    DROP CONSTRAINT IF EXISTS xml_import_history_status_valid;

ALTER TABLE xml_import_history
    ADD CONSTRAINT xml_import_history_status_valid CHECK (
        status IN ('success', 'warning', 'skipped', 'error')
    );
