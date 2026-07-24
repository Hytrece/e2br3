-- Export execution and export-history reads now have separate contextual
-- actions under the existing Export/Submission Edit and View grants. Import
-- history detail reads also enforce the request principal's Case scope.
UPDATE authorization_catalog_state
SET schema_version = 3,
    catalog_hash = '769743e82aba1c42102d6c2afa4fdf4c247d7753f6c613f8274cd0669b26994f',
    reconciled_at = now()
WHERE singleton
  AND catalog_hash = 'c26d5ff8651378fb41b725593cbe755f7bd0e290ce06e5c6dc030b74e96f4c58';
