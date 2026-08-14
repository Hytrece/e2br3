-- E2B Date/Time fields are HL7 TS, not SQL DATE. Preserve precision, time and offset.
ALTER TABLE safety_report_identification
    ALTER COLUMN date_first_received_from_source TYPE TEXT
        USING CASE WHEN date_first_received_from_source IS NULL THEN NULL
                   ELSE replace(date_first_received_from_source::text, '-', '') END,
    ALTER COLUMN date_of_most_recent_information TYPE TEXT
        USING CASE WHEN date_of_most_recent_information IS NULL THEN NULL
                   ELSE replace(date_of_most_recent_information::text, '-', '') END;

ALTER TABLE reactions
    ALTER COLUMN start_date TYPE TEXT
        USING CASE WHEN start_date IS NULL THEN NULL
                   ELSE replace(start_date::text, '-', '') END,
    ALTER COLUMN end_date TYPE TEXT
        USING CASE WHEN end_date IS NULL THEN NULL
                   ELSE replace(end_date::text, '-', '') END;

ALTER TABLE medical_history_episodes
    ALTER COLUMN start_date TYPE TEXT
        USING CASE WHEN start_date IS NULL THEN NULL
                   ELSE replace(start_date::text, '-', '') END,
    ALTER COLUMN end_date TYPE TEXT
        USING CASE WHEN end_date IS NULL THEN NULL
                   ELSE replace(end_date::text, '-', '') END;
