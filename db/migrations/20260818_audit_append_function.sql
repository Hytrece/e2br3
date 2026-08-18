-- Keep explicit domain audit events on a DB-owned append path. Fresh bootstrap
-- row triggers use the same primitive.
CREATE OR REPLACE FUNCTION append_audit_log(
    p_table_name TEXT,
    p_record_id UUID,
    p_organization_id UUID,
    p_action TEXT,
    p_old_values JSONB,
    p_new_values JSONB,
    p_changed_fields JSONB
)
RETURNS VOID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    v_user_id UUID;
BEGIN
    v_user_id := get_current_user_context();

    INSERT INTO audit_logs (
        table_name,
        record_id,
        organization_id,
        action,
        user_id,
        reason_for_change,
        change_category,
        e_signature_id,
        old_values,
        new_values,
        changed_fields
    )
    VALUES (
        p_table_name,
        p_record_id,
        p_organization_id,
        p_action,
        v_user_id,
        get_current_change_reason(),
        get_current_change_category(),
        get_current_esignature_id(),
        p_old_values,
        p_new_values,
        p_changed_fields
    );
EXCEPTION
    WHEN OTHERS THEN
        RAISE EXCEPTION 'Audit trail logging failed for table %.%: %. User context may not be set.',
            p_table_name, p_record_id, SQLERRM;
END;
$$;

REVOKE ALL ON FUNCTION append_audit_log(TEXT, UUID, UUID, TEXT, JSONB, JSONB, JSONB)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION append_audit_log(TEXT, UUID, UUID, TEXT, JSONB, JSONB, JSONB)
    TO e2br3_app_role;
