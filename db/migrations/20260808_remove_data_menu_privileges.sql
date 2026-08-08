UPDATE permission_profiles
SET privileges_json = COALESCE(
    (
        SELECT jsonb_agg(privilege ORDER BY ordinal)
        FROM jsonb_array_elements(privileges_json) WITH ORDINALITY AS rows(privilege, ordinal)
        WHERE lower(trim(privilege ->> 'menu_key')) <> 'data'
    ),
    '[]'::jsonb
)
WHERE jsonb_typeof(privileges_json) = 'array'
AND EXISTS (
    SELECT 1
    FROM jsonb_array_elements(privileges_json) AS privilege
    WHERE lower(trim(privilege ->> 'menu_key')) = 'data'
);
