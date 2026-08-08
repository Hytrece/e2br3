-- Permit one login identity per organization while preserving tenant-local
-- uniqueness. Existing globally unique constraints are replaced after the
-- bootstrap data has been loaded.
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_email_key;
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_username_key;

CREATE UNIQUE INDEX IF NOT EXISTS users_organization_email_unique
    ON users (organization_id, lower(btrim(email)));
CREATE UNIQUE INDEX IF NOT EXISTS users_organization_username_unique
    ON users (organization_id, lower(btrim(username)));

DROP POLICY IF EXISTS users_org_isolation_select ON users;
CREATE POLICY users_org_isolation_select ON users
    FOR SELECT
    TO e2br3_app_role
    USING (
        organization_id = current_organization_id()
        OR is_current_user_admin()
        OR lower(btrim(email)) = lower(btrim(current_setting('app.auth_email', true)))
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
        OR EXISTS (
            SELECT 1
            FROM users same_email_user
            WHERE same_email_user.organization_id = organizations.id
              AND lower(btrim(same_email_user.email)) = lower(btrim(current_setting('app.auth_email', true)))
              AND same_email_user.active = true
        )
    );

DROP POLICY IF EXISTS user_organization_memberships_read
    ON user_organization_memberships;
CREATE POLICY user_organization_memberships_read
    ON user_organization_memberships
    FOR SELECT
    TO e2br3_app_role
    USING (
        user_id = NULLIF(current_setting('app.current_user_id', true), '')::UUID
        OR is_current_user_admin()
    );
