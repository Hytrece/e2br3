CREATE OR REPLACE FUNCTION authz_lock_policy_revisions(
	target_user_id uuid,
	target_organization_id uuid
) RETURNS TABLE (
	organization_revision bigint,
	principal_revision bigint
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path FROM CURRENT
AS $$
DECLARE
	request_user_id uuid;
	request_organization_id uuid;
BEGIN
	request_user_id :=
		NULLIF(current_setting('app.current_user_id', true), '')::uuid;
	request_organization_id :=
		NULLIF(current_setting('app.current_organization_id', true), '')::uuid;
	IF request_user_id IS DISTINCT FROM target_user_id
	   OR request_organization_id IS DISTINCT FROM target_organization_id THEN
		RAISE EXCEPTION 'authorization revision lock is not bound to this request'
			USING ERRCODE = '42501';
	END IF;

	RETURN QUERY
	SELECT organization_state.revision, principal_state.revision
	  FROM organization_policy_state organization_state
	  JOIN principal_authorization_state principal_state
	    ON principal_state.organization_id =
	       organization_state.organization_id
	 WHERE organization_state.organization_id = target_organization_id
	   AND principal_state.user_id = target_user_id
	 FOR UPDATE OF organization_state, principal_state;
END;
$$;

REVOKE ALL ON FUNCTION authz_lock_policy_revisions(uuid, uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION authz_lock_policy_revisions(uuid, uuid)
	TO e2br3_app_role;
