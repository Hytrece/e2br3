-- Seriousness criteria can carry either a concrete boolean or a NullFlavor.
-- Nullable booleans preserve the distinction between false and not provided.
ALTER TABLE reactions
    ALTER COLUMN criteria_death DROP NOT NULL,
    ALTER COLUMN criteria_death DROP DEFAULT,
    ALTER COLUMN criteria_life_threatening DROP NOT NULL,
    ALTER COLUMN criteria_life_threatening DROP DEFAULT,
    ALTER COLUMN criteria_hospitalization DROP NOT NULL,
    ALTER COLUMN criteria_hospitalization DROP DEFAULT,
    ALTER COLUMN criteria_disabling DROP NOT NULL,
    ALTER COLUMN criteria_disabling DROP DEFAULT,
    ALTER COLUMN criteria_congenital_anomaly DROP NOT NULL,
    ALTER COLUMN criteria_congenital_anomaly DROP DEFAULT,
    ALTER COLUMN criteria_other_medically_important DROP NOT NULL,
    ALTER COLUMN criteria_other_medically_important DROP DEFAULT;
