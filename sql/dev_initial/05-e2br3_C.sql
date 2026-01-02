-- ============================================================================
-- SECTION C: Safety Report Identification
-- ============================================================================

CREATE TABLE safety_report_identification (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,

    -- C.1.2 - Date of Creation (MANDATORY)
    transmission_date DATE NOT NULL,

    -- C.1.3 - Type of Report (MANDATORY - E2B(R3) codes)
    report_type VARCHAR(1) NOT NULL CHECK (report_type IN ('1', '2', '3', '4')),
    -- 1=Spontaneous report, 2=Report from study, 3=Other, 4=Not available

    -- C.1.4 - Date Report Was First Received from Source (MANDATORY)
    date_first_received_from_source DATE NOT NULL,

    -- C.1.5 - Date of Most Recent Information (MANDATORY)
    date_of_most_recent_information DATE NOT NULL,

    -- C.1.7 - Fulfils Expedited Criteria (MANDATORY)
    fulfil_expedited_criteria BOOLEAN NOT NULL,

    -- C.1.8.1 - Worldwide Unique Case Identification
    worldwide_unique_id VARCHAR(100),

    -- C.1.10.r - Linked Report Numbers (handled in separate table)

    -- C.1.11.2 - Nullification Reason
    nullification_reason TEXT,

    -- Receiver Organization
    receiver_organization VARCHAR(200),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_identification_per_case UNIQUE (case_id)
);

CREATE INDEX idx_safety_report_id_case ON safety_report_identification(case_id);
CREATE INDEX idx_safety_report_id_worldwide ON safety_report_identification(worldwide_unique_id);

-- ============================================================================
-- SECTION C.3: Sender Information (MANDATORY fields)
-- ============================================================================

CREATE TABLE sender_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,

    -- C.3.1 - Sender Type (MANDATORY - E2B(R3) codes)
    sender_type VARCHAR(1) NOT NULL CHECK (sender_type IN ('1', '2', '3', '4', '5', '6')),
    -- 1=Pharmaceutical company, 2=Regulatory authority, 3=Health professional
    -- 4=Regional pharmacovigilance center, 5=WHO collaborating center, 6=Other

    -- C.3.2 - Sender's Organisation (MANDATORY)
    organization_name VARCHAR(200) NOT NULL,
    department VARCHAR(100),
    street_address VARCHAR(100),
    city VARCHAR(50),
    state VARCHAR(40),
    postcode VARCHAR(15),
    country_code VARCHAR(2),  -- ISO 3166-1 alpha-2

    -- C.3.3 - Person Responsible for Sending
    person_title VARCHAR(50),
    person_given_name VARCHAR(60),
    person_middle_name VARCHAR(60),
    person_family_name VARCHAR(60),

    -- C.3.4 - Contact Information
    telephone VARCHAR(33),
    fax VARCHAR(33),
    email VARCHAR(100),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_sender_per_case UNIQUE (case_id)
);

CREATE INDEX idx_sender_info_case ON sender_information(case_id);

-- ============================================================================
-- SECTION C.4.r: Literature References (Repeating)
-- ============================================================================

CREATE TABLE literature_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    reference_text TEXT NOT NULL,  -- C.4.r
    sequence_number INTEGER NOT NULL,  -- For ordering
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_lit_ref_sequence UNIQUE (case_id, sequence_number)
);

CREATE INDEX idx_literature_refs_case ON literature_references(case_id);

-- ============================================================================
-- SECTION C.5: Study Information
-- ============================================================================

CREATE TABLE study_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,

    -- C.5.1 - Study Name
    study_name VARCHAR(500),

    -- C.5.2 - Sponsor Study Number
    sponsor_study_number VARCHAR(100),

    -- C.5.3 - Study Type Reaction
    study_type_reaction VARCHAR(2),  -- E2B(R3) code list

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_study_per_case UNIQUE (case_id)
);

-- C.5.1.r - Study Registration Numbers (Repeating)
CREATE TABLE study_registration_numbers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    study_information_id UUID NOT NULL REFERENCES study_information(id) ON DELETE CASCADE,
    registration_number VARCHAR(100) NOT NULL,
    country_code VARCHAR(2),  -- ISO 3166-1 alpha-2
    sequence_number INTEGER NOT NULL,

    CONSTRAINT unique_study_reg_num UNIQUE (study_information_id, sequence_number)
);

CREATE INDEX idx_study_info_case ON study_information(case_id);
CREATE INDEX idx_study_reg_nums ON study_registration_numbers(study_information_id);

-- ============================================================================
-- SECTION C.2.r: Primary Source Information (Repeating)
-- ============================================================================

CREATE TABLE primary_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,  -- For ordering multiple reporters

    -- C.2.r.1 - Reporter's Name
    reporter_title VARCHAR(50),
    reporter_given_name VARCHAR(60),
    reporter_middle_name VARCHAR(60),
    reporter_family_name VARCHAR(60),

    -- C.2.r.2 - Reporter's Address and Contact
    organization VARCHAR(60),
    department VARCHAR(60),
    street VARCHAR(100),
    city VARCHAR(35),
    state VARCHAR(40),
    postcode VARCHAR(15),
    telephone VARCHAR(33),

    -- C.2.r.3 - Country Code
    country_code VARCHAR(2),  -- ISO 3166-1 alpha-2

    -- Email (not in spec but commonly used)
    email VARCHAR(100),

    -- C.2.r.4 - Qualification (MANDATORY within primary source - E2B(R3) codes)
    qualification VARCHAR(1) CHECK (qualification IN ('1', '2', '3', '4', '5')),
    -- 1=Physician, 2=Pharmacist, 3=Other health professional, 4=Lawyer, 5=Consumer

    -- C.2.r.5 - Primary Source for Regulatory Purposes (MANDATORY)
    primary_source_regulatory VARCHAR(1) CHECK (primary_source_regulatory IN ('1', '2', '3')),
    -- 1=Yes, 2=No, 3=Unknown

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_primary_source_sequence UNIQUE (case_id, sequence_number)
);

CREATE INDEX idx_primary_sources_case ON primary_sources(case_id);