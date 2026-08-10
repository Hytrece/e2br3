CREATE OR REPLACE FUNCTION public.audit_parent_record_ids(
    p_old_values JSONB,
    p_new_values JSONB
)
RETURNS UUID[]
LANGUAGE SQL
IMMUTABLE
PARALLEL SAFE
SET search_path = pg_catalog
AS $$
    SELECT ARRAY(
        SELECT DISTINCT field.value::UUID
        FROM pg_catalog.jsonb_each_text(
            COALESCE(p_new_values, '{}'::JSONB)
            || COALESCE(p_old_values, '{}'::JSONB)
        ) AS field
        WHERE field.key = ANY(ARRAY[
            'case_id', 'death_info_id', 'device_id', 'drug_id',
            'drug_reaction_assessment_id', 'e_signature_id',
            'narrative_id', 'parent_id', 'patient_id', 'reaction_id',
            'study_information_id', 'submission_id'
        ])
          AND field.value ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
    );
$$;

DROP INDEX IF EXISTS idx_audit_logs_parent_record_ids;

ALTER TABLE audit_logs
    ADD COLUMN IF NOT EXISTS parent_record_ids UUID[]
    GENERATED ALWAYS AS (
        public.audit_parent_record_ids(old_values, new_values)
    ) STORED;

CREATE INDEX idx_audit_logs_parent_record_ids
    ON audit_logs USING GIN (parent_record_ids);
