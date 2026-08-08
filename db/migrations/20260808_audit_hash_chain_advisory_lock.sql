-- Replace the audit hash-chain relation lock with a transaction-scoped
-- advisory lock.  The old SHARE ROW EXCLUSIVE relation lock could deadlock
-- when a concurrent audited write held a source-table lock first.
CREATE OR REPLACE FUNCTION audit_logs_hash_chain_before_insert()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_prev_hash TEXT;
    v_payload TEXT;
BEGIN
    IF NEW.created_at IS NULL THEN
        NEW.created_at := NOW();
    END IF;

    -- Keep hash-chain appends serialized without taking a heavyweight
    -- relation lock that participates in source-table lock cycles.
    PERFORM pg_advisory_xact_lock(
        hashtextextended('e2br3.audit_logs.hash_chain', 0)
    );

    SELECT entry_hash
      INTO v_prev_hash
      FROM audit_logs
     ORDER BY id DESC
     LIMIT 1;

    NEW.prev_hash := COALESCE(v_prev_hash, repeat('0', 64));

    v_payload := concat_ws(
        '|',
        COALESCE(NEW.id::TEXT, ''),
        NEW.prev_hash,
        NEW.table_name,
        NEW.record_id::TEXT,
        NEW.action,
        NEW.user_id::TEXT,
        COALESCE(NEW.reason_for_change, ''),
        COALESCE(NEW.change_category, ''),
        COALESCE(NEW.e_signature_id::TEXT, ''),
        COALESCE(NEW.old_values::TEXT, 'null'),
        COALESCE(NEW.new_values::TEXT, 'null'),
        COALESCE(NEW.changed_fields::TEXT, 'null'),
        COALESCE(NEW.ip_address::TEXT, ''),
        COALESCE(NEW.user_agent, ''),
        to_char(NEW.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')
    );

    NEW.entry_hash := encode(digest(v_payload, 'sha256'), 'hex');
    RETURN NEW;
END;
$$;
