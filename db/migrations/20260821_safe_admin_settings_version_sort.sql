-- Terminology test and vendor releases may use non-numeric version labels.
CREATE OR REPLACE FUNCTION app_settings_default_value()
RETURNS JSONB
LANGUAGE SQL
STABLE
AS $$
    WITH default_meddra AS (
        SELECT
            CASE language
                WHEN 'en' THEN 'English'
                WHEN 'ko' THEN 'Korean'
                ELSE initcap(language)
            END AS language_label,
            version
        FROM terminology_releases
        WHERE dictionary = 'meddra'
          AND status = 'active'
        ORDER BY
            (language = 'en' AND version = '28.1') DESC,
            (language = 'en') DESC,
            CASE
                WHEN version ~ '^[0-9]+(\.[0-9]+)*$'
                    THEN string_to_array(version, '.')::int[]
                ELSE ARRAY[]::int[]
            END DESC,
            activated_at DESC NULLS LAST,
            updated_at DESC
        LIMIT 1
    )
    SELECT jsonb_build_object(
        'timezone', 'Asia/Seoul',
        'meddra_language', COALESCE(
            (SELECT language_label FROM default_meddra),
            'English'
        ),
        'meddra_version', COALESCE(
            (SELECT version FROM default_meddra),
            '28.1'
        ),
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
