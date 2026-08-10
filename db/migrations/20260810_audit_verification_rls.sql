

CREATE OR REPLACE FUNCTION verify_audit_log_hash_chain(p_since_id BIGINT DEFAULT NULL)
RETURNS TABLE (
    total_rows BIGINT,
    verified_ok_rows BIGINT,
    broken_rows BIGINT,
    first_broken_id BIGINT,
    first_broken_reason TEXT,
    checked_at TIMESTAMPTZ
)
LANGUAGE sql
SECURITY DEFINER
SET search_path = pg_catalog
AS $$
    WITH chain AS (
        SELECT id,
               prev_hash,
               entry_hash,
               lag(entry_hash) OVER (ORDER BY id) AS expected_prev_hash,
               encode(
                   public.digest(
                       concat_ws(
                           '|',
                           coalesce(id::text, ''),
                           coalesce(prev_hash, ''),
                           table_name,
                           record_id::text,
                           action,
                           user_id::text,
                           coalesce(reason_for_change, ''),
                           coalesce(change_category, ''),
                           coalesce(e_signature_id::text, ''),
                           coalesce(old_values::text, 'null'),
                           coalesce(new_values::text, 'null'),
                           coalesce(changed_fields::text, 'null'),
                           coalesce(ip_address::text, ''),
                           coalesce(user_agent, ''),
                           to_char(
                               created_at AT TIME ZONE 'UTC',
                               'YYYY-MM-DD"T"HH24:MI:SS.US"Z"'
                           )
                       ),
                       'sha256'
                   ),
                   'hex'
               ) AS expected_entry_hash
          FROM public.audit_logs
    ),
    checked AS (
        SELECT id,
               CASE
                   WHEN prev_hash !~ '^[0-9A-Fa-f]{64}$'
                       THEN 'prev_hash is not a 64-char hex value'
                   WHEN entry_hash !~ '^[0-9A-Fa-f]{64}$'
                       THEN 'entry_hash is not a 64-char hex value'
                   WHEN prev_hash <> coalesce(expected_prev_hash, repeat('0', 64))
                       THEN 'prev_hash does not match previous entry_hash'
                   WHEN entry_hash <> expected_entry_hash
                       THEN 'entry_hash does not match recomputed payload hash'
               END AS reason
          FROM chain
         WHERE p_since_id IS NULL OR id >= p_since_id
    )
    SELECT count(*)::bigint,
           count(*) FILTER (WHERE reason IS NULL)::bigint,
           count(*) FILTER (WHERE reason IS NOT NULL)::bigint,
           min(id) FILTER (WHERE reason IS NOT NULL),
           (array_agg(reason ORDER BY id) FILTER (WHERE reason IS NOT NULL))[1],
           clock_timestamp()
      FROM checked;
$$;

REVOKE ALL ON FUNCTION verify_audit_log_hash_chain(BIGINT) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION verify_audit_log_hash_chain(BIGINT) TO e2br3_app_role;
