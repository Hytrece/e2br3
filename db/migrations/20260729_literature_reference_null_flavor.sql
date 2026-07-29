-- C.4.r.1 carries either a concrete literature reference or a NullFlavor.
-- The former NOT NULL column forced an empty-string sentinel and conflicted
-- with the database-wide value/NullFlavor exclusivity invariant.
ALTER TABLE literature_references
    ALTER COLUMN reference_text DROP NOT NULL;
