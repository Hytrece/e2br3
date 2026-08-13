DROP POLICY IF EXISTS meddra_terms_read ON meddra_terms;
CREATE POLICY meddra_terms_read ON meddra_terms
    FOR SELECT TO e2br3_app_role
    USING (active = true OR is_current_user_admin());

DROP POLICY IF EXISTS whodrug_products_read ON whodrug_products;
CREATE POLICY whodrug_products_read ON whodrug_products
    FOR SELECT TO e2br3_app_role
    USING (active = true OR is_current_user_admin());

ALTER TABLE terminology_releases ENABLE ROW LEVEL SECURITY;
ALTER TABLE terminology_releases FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS terminology_releases_read ON terminology_releases;
CREATE POLICY terminology_releases_read ON terminology_releases
    FOR SELECT TO e2br3_app_role
    USING (status IN ('validated', 'approved', 'active') OR is_current_user_admin());
DROP POLICY IF EXISTS terminology_releases_insert ON terminology_releases;
CREATE POLICY terminology_releases_insert ON terminology_releases
    FOR INSERT TO e2br3_app_role
    WITH CHECK (is_current_user_admin());
DROP POLICY IF EXISTS terminology_releases_update ON terminology_releases;
CREATE POLICY terminology_releases_update ON terminology_releases
    FOR UPDATE TO e2br3_app_role
    USING (is_current_user_admin())
    WITH CHECK (is_current_user_admin());
DROP POLICY IF EXISTS terminology_releases_delete ON terminology_releases;
CREATE POLICY terminology_releases_delete ON terminology_releases
    FOR DELETE TO e2br3_app_role
    USING (is_current_user_admin());
