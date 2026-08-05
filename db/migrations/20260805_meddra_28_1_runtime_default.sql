-- Select the licensed MedDRA 28.1 release for existing organizations that
-- still carry the old runtime default.
UPDATE app_settings
SET value = jsonb_set(value, '{meddra_version}', '"28.1"'::jsonb, true),
    updated_at = NOW()
WHERE key = 'system'
  AND value->>'meddra_version' = '26.0';
