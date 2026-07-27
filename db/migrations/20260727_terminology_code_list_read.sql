DROP POLICY IF EXISTS e2b_code_lists_read ON e2b_code_lists;

CREATE POLICY e2b_code_lists_read ON e2b_code_lists
    FOR SELECT TO e2br3_app_role
    USING (active = true OR is_current_user_admin());
