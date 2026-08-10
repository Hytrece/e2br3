-- Keep one account per email. Organization access belongs to memberships.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM users
        GROUP BY lower(btrim(email))
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION
            'duplicate user email accounts must be merged before global identity migration';
    END IF;
    IF EXISTS (
        SELECT 1
        FROM users
        GROUP BY lower(btrim(username))
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION
            'duplicate user usernames must be merged before global identity migration';
    END IF;
END $$;

INSERT INTO user_organization_memberships (
    user_id, organization_id, active, created_by, updated_by
)
SELECT id, organization_id, COALESCE(active, true),
       COALESCE(created_by, id), updated_by
FROM users
WHERE organization_id IS NOT NULL
ON CONFLICT (user_id, organization_id) DO UPDATE
SET active = EXCLUDED.active,
    updated_by = EXCLUDED.updated_by,
    updated_at = NOW();

DROP INDEX IF EXISTS users_organization_email_unique;
DROP INDEX IF EXISTS users_organization_username_unique;
CREATE UNIQUE INDEX IF NOT EXISTS users_email_unique
    ON users (lower(btrim(email)));
CREATE UNIQUE INDEX IF NOT EXISTS users_username_unique
    ON users (lower(btrim(username)));

DROP POLICY IF EXISTS users_org_isolation_select ON users;
CREATE POLICY users_org_isolation_select ON users
    FOR SELECT
    TO e2br3_app_role
    USING (
        organization_id = current_organization_id()
        OR is_current_user_admin()
        OR id = NULLIF(current_setting('app.current_user_id', true), '')::UUID
    );

DROP POLICY IF EXISTS orgs_select ON organizations;
CREATE POLICY orgs_select ON organizations
    FOR SELECT
    TO e2br3_app_role
    USING (
        id = current_organization_id()
        OR is_current_user_admin()
        OR EXISTS (
            SELECT 1
            FROM user_organization_memberships membership
            WHERE membership.organization_id = organizations.id
              AND membership.user_id = NULLIF(current_setting('app.current_user_id', true), '')::UUID
              AND membership.active = true
        )
    );
