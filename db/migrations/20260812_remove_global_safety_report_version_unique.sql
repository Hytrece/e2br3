-- C.1.1/version uniqueness is organization-scoped by the import workflow.
-- This index incorrectly rejected the same identifier in different organizations.
DROP INDEX IF EXISTS idx_safety_report_identification_report_version;
