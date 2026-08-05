CREATE TABLE IF NOT EXISTS case_e2b_field_notations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    record_id UUID,
    e2b_code VARCHAR(64) NOT NULL,
    notation TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE NULLS NOT DISTINCT (case_id, record_id, e2b_code)
);

ALTER TABLE case_e2b_field_notations ENABLE ROW LEVEL SECURITY;
ALTER TABLE case_e2b_field_notations FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS case_e2b_field_notations_via_case ON case_e2b_field_notations;
CREATE POLICY case_e2b_field_notations_via_case ON case_e2b_field_notations
    FOR ALL TO e2br3_app_role
    USING (EXISTS (
        SELECT 1 FROM cases c
        WHERE c.id = case_e2b_field_notations.case_id
          AND (c.organization_id = current_organization_id() OR is_current_user_admin())
    ))
    WITH CHECK (EXISTS (
        SELECT 1 FROM cases c
        WHERE c.id = case_e2b_field_notations.case_id
          AND (c.organization_id = current_organization_id() OR is_current_user_admin())
    ));

GRANT SELECT, INSERT, UPDATE, DELETE ON case_e2b_field_notations TO e2br3_app_role;

DROP TRIGGER IF EXISTS update_case_e2b_field_notations_updated_at ON case_e2b_field_notations;
CREATE TRIGGER update_case_e2b_field_notations_updated_at
  BEFORE UPDATE ON case_e2b_field_notations
  FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS audit_case_e2b_field_notations ON case_e2b_field_notations;
CREATE TRIGGER audit_case_e2b_field_notations
  AFTER INSERT OR UPDATE OR DELETE ON case_e2b_field_notations
  FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();
