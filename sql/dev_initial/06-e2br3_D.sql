-- ============================================================================
-- SECTION D: Patient Information
-- ============================================================================

CREATE TABLE patient_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,

    -- D.1 - Patient (name identifier)
    patient_initials VARCHAR(60),
    patient_given_name VARCHAR(60),  -- Can use null flavor
    patient_family_name VARCHAR(60),  -- Can use null flavor

    -- D.2.1 - Date of Birth
    birth_date DATE,  -- Can use null flavor

    -- D.2.2 - Age Information
    age_at_time_of_onset DECIMAL(5,2),
    age_unit VARCHAR(3),  -- E2B(R3) codes: 800-805 (year/month/week/day/hour/decade)

    -- D.2.2.1 - Gestation Period (for fetal cases)
    gestation_period DECIMAL(5,2),
    gestation_period_unit VARCHAR(3),  -- 802, 803, 804 (month, week, day)

    -- D.2.3 - Patient Age Group (E2B(R3) codes)
    age_group VARCHAR(1) CHECK (age_group IN ('1', '2', '3', '4', '5', '6')),
    -- 1=Neonate, 2=Infant, 3=Child, 4=Adolescent, 5=Adult, 6=Elderly

    -- D.3 - Body Weight (kg)
    weight_kg DECIMAL(6,2),

    -- D.4 - Height (cm)
    height_cm DECIMAL(6,2),

    -- D.5 - Sex (E2B(R3) codes)
    sex VARCHAR(1) CHECK (sex IN ('0', '1', '2')),  -- 0=Unknown, 1=Male, 2=Female

    -- D.6 - Last Menstrual Period Date
    last_menstrual_period_date DATE,

    -- D.7.2 - Text for Relevant Medical History
    medical_history_text TEXT,  -- Max 10000 chars

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_patient_per_case UNIQUE (case_id)
);

CREATE INDEX idx_patient_info_case ON patient_information(case_id);

-- ============================================================================
-- D.7.1.r: Medical History Episodes (Repeating)
-- ============================================================================

CREATE TABLE medical_history_episodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patient_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- D.7.1.r.1a - Disease/Surgical Procedure/etc (MedDRA coded)
    meddra_version VARCHAR(10),
    meddra_code VARCHAR(20),

    -- D.7.1.r.2 - Start Date
    start_date DATE,

    -- D.7.1.r.3 - Continuing
    continuing BOOLEAN,

    -- D.7.1.r.4 - End Date
    end_date DATE,

    -- D.7.1.r.5 - Comments
    comments TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_med_history_sequence UNIQUE (patient_id, sequence_number)
);

CREATE INDEX idx_med_history_patient ON medical_history_episodes(patient_id);

-- ============================================================================
-- D.8.r: Past Drug History (Repeating)
-- ============================================================================

CREATE TABLE past_drug_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patient_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- D.8.r.1 - Drug Name
    drug_name VARCHAR(500),

    -- D.8.r.2 - MPID (Medicinal Product ID)
    mpid VARCHAR(100),
    mpid_version VARCHAR(10),

    -- D.8.r.3 - PhPID (Pharmaceutical Product ID)
    phpid VARCHAR(100),
    phpid_version VARCHAR(10),

    -- D.8.r.4 - Start Date
    start_date DATE,

    -- D.8.r.5 - End Date
    end_date DATE,

    -- D.8.r.6a - Indication (MedDRA coded)
    indication_meddra_version VARCHAR(10),
    indication_meddra_code VARCHAR(20),

    -- D.8.r.7 - Reaction(s) (MedDRA coded - repeating, handled separately if needed)

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_past_drug_sequence UNIQUE (patient_id, sequence_number)
);

CREATE INDEX idx_past_drug_patient ON past_drug_history(patient_id);

-- ============================================================================
-- D.9: Death Information
-- ============================================================================

CREATE TABLE patient_death_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patient_information(id) ON DELETE CASCADE,

    -- D.9.1 - Date of Death
    date_of_death DATE,

    -- D.9.3 - Autopsy
    autopsy_performed BOOLEAN,

    -- D.9.4 - Autopsy Determined Cause of Death (handled in repeating table)

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_death_per_patient UNIQUE (patient_id)
);

-- D.9.2.r: Reported Cause(s) of Death (Repeating, MedDRA coded)
CREATE TABLE reported_causes_of_death (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    death_info_id UUID NOT NULL REFERENCES patient_death_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,
    meddra_version VARCHAR(10),
    meddra_code VARCHAR(20),

    CONSTRAINT unique_reported_death_cause UNIQUE (death_info_id, sequence_number)
);

-- D.9.4.r: Autopsy-determined Cause(s) of Death (Repeating, MedDRA coded)
CREATE TABLE autopsy_causes_of_death (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    death_info_id UUID NOT NULL REFERENCES patient_death_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,
    meddra_version VARCHAR(10),
    meddra_code VARCHAR(20),

    CONSTRAINT unique_autopsy_death_cause UNIQUE (death_info_id, sequence_number)
);

CREATE INDEX idx_death_info_patient ON patient_death_information(patient_id);

-- ============================================================================
-- D.10: Parent Information (for fetal/neonatal cases)
-- ============================================================================

CREATE TABLE parent_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES patient_information(id) ON DELETE CASCADE,

    -- D.10.1 - Parent Identification
    parent_identification VARCHAR(60),

    -- D.10.2 - Parent Age
    parent_age DECIMAL(5,2),
    parent_age_unit VARCHAR(3),  -- 800-804 (no hour unit for parents)

    -- D.10.3 - Last Menstrual Period Date
    last_menstrual_period_date DATE,

    -- D.10.4 - Parent Weight (kg)
    weight_kg DECIMAL(6,2),

    -- D.10.5 - Parent Height (cm)
    height_cm DECIMAL(6,2),

    -- D.10.6 - Parent Sex
    sex VARCHAR(1) CHECK (sex IN ('0', '1', '2')),

    -- D.10.7 - Parent Medical History (repeating - uses medical_history_episodes table with parent_id FK)

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_parent_per_patient UNIQUE (patient_id)
);

CREATE INDEX idx_parent_info_patient ON parent_information(patient_id);