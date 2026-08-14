-- E.i.6a is a REAL lexeme; preserve 54 and 54.00 distinctly.
ALTER TABLE reactions
    ALTER COLUMN duration_value TYPE TEXT
        USING duration_value::text;
