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

ALTER FUNCTION audit_logs_hash_chain_before_insert() SET search_path = public;
ALTER FUNCTION audit_log_organization_id(TEXT, UUID, JSONB, JSONB) SET search_path = public;

-- Upgrade existing databases too: bootstrap and migrated installations must
-- send row-triggered writes through the same append primitive.
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    v_old_business JSONB;
    v_new_business JSONB;
    v_changed_fields JSONB;
BEGIN
    PERFORM get_current_user_context();
    IF TG_OP = 'INSERT' THEN
        v_changed_fields := compute_audit_changed_fields(NULL, to_jsonb(NEW));
        PERFORM append_audit_log(TG_TABLE_NAME, NEW.id,
            audit_log_organization_id(TG_TABLE_NAME, NEW.id, NULL, to_jsonb(NEW)),
            'CREATE', NULL, to_jsonb(NEW), v_changed_fields);
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        v_old_business := to_jsonb(OLD) - 'updated_at' - 'updated_by';
        v_new_business := to_jsonb(NEW) - 'updated_at' - 'updated_by';
        IF v_old_business = v_new_business THEN RETURN NEW; END IF;
        v_changed_fields := compute_audit_changed_fields(v_old_business, v_new_business);
        PERFORM append_audit_log(TG_TABLE_NAME, NEW.id,
            audit_log_organization_id(TG_TABLE_NAME, NEW.id, to_jsonb(OLD), to_jsonb(NEW)),
            'UPDATE', to_jsonb(OLD), to_jsonb(NEW), v_changed_fields);
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        v_changed_fields := compute_audit_changed_fields(to_jsonb(OLD), NULL);
        PERFORM append_audit_log(TG_TABLE_NAME, OLD.id,
            audit_log_organization_id(TG_TABLE_NAME, OLD.id, to_jsonb(OLD), NULL),
            'DELETE', to_jsonb(OLD), NULL, v_changed_fields);
        RETURN OLD;
    END IF;
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Audit trail logging failed for table %.%: %. User context may not be set.',
        TG_TABLE_SCHEMA, TG_TABLE_NAME, SQLERRM;
END;
$$;

CREATE OR REPLACE FUNCTION audit_trigger_function_with_audit_id()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    v_old_business JSONB;
    v_new_business JSONB;
    v_changed_fields JSONB;
BEGIN
    PERFORM get_current_user_context();
    IF TG_OP = 'INSERT' THEN
        v_changed_fields := compute_audit_changed_fields(NULL, to_jsonb(NEW));
        PERFORM append_audit_log(TG_TABLE_NAME, NEW.audit_id,
            audit_log_organization_id(TG_TABLE_NAME, NEW.audit_id, NULL, to_jsonb(NEW)),
            'CREATE', NULL, to_jsonb(NEW), v_changed_fields);
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        v_old_business := to_jsonb(OLD) - 'updated_at' - 'updated_by';
        v_new_business := to_jsonb(NEW) - 'updated_at' - 'updated_by';
        IF v_old_business = v_new_business THEN RETURN NEW; END IF;
        v_changed_fields := compute_audit_changed_fields(v_old_business, v_new_business);
        PERFORM append_audit_log(TG_TABLE_NAME, NEW.audit_id,
            audit_log_organization_id(TG_TABLE_NAME, NEW.audit_id, to_jsonb(OLD), to_jsonb(NEW)),
            'UPDATE', to_jsonb(OLD), to_jsonb(NEW), v_changed_fields);
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        v_changed_fields := compute_audit_changed_fields(to_jsonb(OLD), NULL);
        PERFORM append_audit_log(TG_TABLE_NAME, OLD.audit_id,
            audit_log_organization_id(TG_TABLE_NAME, OLD.audit_id, to_jsonb(OLD), NULL),
            'DELETE', to_jsonb(OLD), NULL, v_changed_fields);
        RETURN OLD;
    END IF;
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Audit trail logging failed for table %.%: %. User context may not be set.',
        TG_TABLE_SCHEMA, TG_TABLE_NAME, SQLERRM;
END;
$$;

CREATE OR REPLACE FUNCTION audit_trigger_function_with_submission_id()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    v_old_business JSONB;
    v_new_business JSONB;
    v_changed_fields JSONB;
BEGIN
    PERFORM get_current_user_context();
    IF TG_OP = 'INSERT' THEN
        v_changed_fields := compute_audit_changed_fields(NULL, to_jsonb(NEW));
        PERFORM append_audit_log(TG_TABLE_NAME, NEW.submission_id,
            audit_log_organization_id(TG_TABLE_NAME, NEW.submission_id, NULL, to_jsonb(NEW)),
            'CREATE', NULL, to_jsonb(NEW), v_changed_fields);
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        v_old_business := to_jsonb(OLD) - 'updated_at' - 'updated_by';
        v_new_business := to_jsonb(NEW) - 'updated_at' - 'updated_by';
        IF v_old_business = v_new_business THEN RETURN NEW; END IF;
        v_changed_fields := compute_audit_changed_fields(v_old_business, v_new_business);
        PERFORM append_audit_log(TG_TABLE_NAME, NEW.submission_id,
            audit_log_organization_id(TG_TABLE_NAME, NEW.submission_id, to_jsonb(OLD), to_jsonb(NEW)),
            'UPDATE', to_jsonb(OLD), to_jsonb(NEW), v_changed_fields);
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        v_changed_fields := compute_audit_changed_fields(to_jsonb(OLD), NULL);
        PERFORM append_audit_log(TG_TABLE_NAME, OLD.submission_id,
            audit_log_organization_id(TG_TABLE_NAME, OLD.submission_id, to_jsonb(OLD), NULL),
            'DELETE', to_jsonb(OLD), NULL, v_changed_fields);
        RETURN OLD;
    END IF;
EXCEPTION WHEN OTHERS THEN
    RAISE EXCEPTION 'Audit trail logging failed for table %.%: %. User context may not be set.',
        TG_TABLE_SCHEMA, TG_TABLE_NAME, SQLERRM;
END;
$$;
