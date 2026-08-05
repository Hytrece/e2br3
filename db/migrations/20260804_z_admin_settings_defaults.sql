-- Canonical settings document used by organization creation and backfills.
CREATE OR REPLACE FUNCTION app_settings_default_value()
RETURNS JSONB
LANGUAGE SQL
IMMUTABLE
AS $$
    SELECT jsonb_build_object(
        'timezone', 'Asia/Seoul',
        'meddra_language', 'English',
        'meddra_version', '28.1',
        'idf_version', '3.0',
        'orientation', 'Landscape',
        'data_ordering', 'Primary data will appear first',
        'upload_excel_template_without_element_label', false,
        'notation', false,
        'apply_comments_on_exported_xml', false,
        'apply_sender_info_to_imported_cases', false,
        'import_date_update', jsonb_build_object(
            'date_of_creation', false,
            'most_recent_info_date', false,
            'report_first_received_date', false
        ),
        'appendices', jsonb_build_array('ICH'),
        'case_number_setting', 'AE Row No.',
        'case_number_identifier', 'ICSR',
        'case_number_padding', 6,
        'case_number_sequence_condition', 'Per sender',
        'case_number_format_fields', jsonb_build_array('AE Row No.'),
        'workflow_enabled', false,
        'workflow', jsonb_build_object(
            'statuses', jsonb_build_array(jsonb_build_object(
                'name', 'Saved',
                'editable', true,
                'description', 'Default state',
                'due_days', 0,
                'allowed_roles', jsonb_build_array('sponsor_admin_cro')
            ))
        ),
        'idle_session_minutes', 60,
        'session_warning_minutes', 5
    );
$$;

-- Backfill the strict runtime settings document for existing tenants.
-- New organizations are initialized by OrganizationBmc::create; this closes
-- the gap for tenants created before that path existed.
DO $$
BEGIN
    PERFORM set_config('app.current_user_id', '00000000-0000-0000-0000-000000000001', true);
    PERFORM set_config('app.current_organization_id', '00000000-0000-0000-0000-000000000000', true);
    PERFORM set_config('app.platform_isolation_bypass', 'true', true);

    INSERT INTO app_settings (organization_id, key, value, updated_by)
    SELECT id, 'system', app_settings_default_value(), NULL
    FROM organizations
    WHERE NOT EXISTS (
        SELECT 1
        FROM app_settings existing
        WHERE existing.organization_id = organizations.id
          AND existing.key = 'system'
    );
END
$$;
