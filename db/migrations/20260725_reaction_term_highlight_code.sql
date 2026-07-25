ALTER TABLE reactions
    ALTER COLUMN term_highlighted TYPE VARCHAR(1)
    USING CASE
        WHEN term_highlighted IS TRUE THEN '1'
        WHEN term_highlighted IS FALSE THEN '2'
        ELSE NULL
    END;

ALTER TABLE reactions
    ADD CONSTRAINT reactions_term_highlighted_code
    CHECK (term_highlighted IS NULL OR term_highlighted IN ('1', '2', '3', '4'));
