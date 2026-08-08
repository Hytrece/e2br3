-- E.i.1.1a is optional; code-only MedDRA reactions are valid.
ALTER TABLE reactions
    ALTER COLUMN primary_source_reaction DROP NOT NULL;
