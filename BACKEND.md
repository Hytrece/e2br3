# BACKEND.md

Documentation for the **Rust backend** that serves the E2BR3 SafetyDB Next.js frontend.

## Project Overview

The E2BR3 SafetyDB backend is a **Rust-based REST API** that provides data persistence, authentication, XML processing, and E2B(R3) validation for adverse event case safety reports following the ICH protocol.

**Tech Stack**:
- **Language**: Rust (latest stable)
- **Web Framework**: Axum or Actix-web
- **Database**: PostgreSQL with SQLx or Diesel ORM
- **Authentication**: JWT with bcrypt password hashing
- **XML Processing**: quick-xml or xml-rs with XSD validation
- **Validation**: validator crate for business rules

## Architecture

### Backend Responsibilities

1. **Data Persistence**
   - PostgreSQL database for all E2B(R3) case data
   - Complete schema covering all ICH sections (A-H)
   - MedDRA and WHODrug controlled terminology tables
   - Audit trail and version history for regulatory compliance

2. **Authentication & Authorization**
   - JWT-based authentication with refresh tokens
   - Role-based access control (RBAC)
   - Organization-level data isolation
   - Session management

3. **Business Logic**
   - E2B(R3) validation rules and constraints
   - Cardinality checking (min/max occurrences)
   - Controlled terminology validation
   - Case workflow management (draft, submitted, etc.)

4. **XML Processing**
   - E2B(R3) XML generation with XSD validation
   - XML import with schema validation
   - ICH message header generation
   - Regional requirement handling (FDA/EMA/PMDA)

5. **API Endpoints**
   - RESTful API for frontend consumption
   - File upload for XML imports
   - Terminology lookup endpoints
   - Reporting and analytics endpoints

## Database Architecture

### Database Choice: PostgreSQL

**Decision**: Full relational schema using PostgreSQL (no JSONB)

**Rationale**:
- Regulatory compliance and industry standard
- Strong referential integrity for MedDRA/WHODrug lookups
- Robust audit trail capabilities
- ACID compliance for safety-critical data
- Excellent query performance with proper indexing

### Schema Design Principles

1. **Normalization**: Full 3NF for data integrity
2. **Foreign Keys**: Enforce referential integrity at database level
3. **Null Flavors**: Support E2B(R3) null flavor codes (ASKU, NASK, MSK, etc.)
4. **Repeating Sections**: Separate tables with case_id FK for 1-to-many relationships
5. **Audit Trail**: Comprehensive logging for regulatory compliance
6. **Version History**: Full case versioning support

## Complete Database Schema

### 1. Core System Tables

```sql
-- ============================================================================
-- Organizations and Users
-- ============================================================================

CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(500) NOT NULL,
    type VARCHAR(100),  -- pharma, regulatory, healthcare, etc.
    address TEXT,
    city VARCHAR(200),
    state VARCHAR(100),
    postcode VARCHAR(50),
    country_code VARCHAR(2),  -- ISO 3166-1 alpha-2
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    active BOOLEAN DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    CONSTRAINT user_role_valid CHECK (role IN ('admin', 'manager', 'user', 'viewer'))
);

CREATE INDEX idx_users_organization ON users(organization_id);
CREATE INDEX idx_users_email ON users(email);

-- ============================================================================
-- Cases (Main Table)
-- ============================================================================

CREATE TABLE cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,

    -- Case identification
    safety_report_id VARCHAR(100) NOT NULL,  -- C.1.1
    version INTEGER NOT NULL DEFAULT 1,      -- C.1.1.r.1
    status VARCHAR(50) NOT NULL DEFAULT 'draft',

    -- Workflow tracking
    created_by UUID NOT NULL REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    submitted_by UUID REFERENCES users(id),
    submitted_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint: one active version per safety_report_id
    CONSTRAINT unique_safety_report_version UNIQUE (safety_report_id, version),
    CONSTRAINT case_status_valid CHECK (status IN ('draft', 'validated', 'submitted', 'archived', 'nullified'))
);

CREATE INDEX idx_cases_organization ON cases(organization_id);
CREATE INDEX idx_cases_safety_report_id ON cases(safety_report_id);
CREATE INDEX idx_cases_status ON cases(status);
CREATE INDEX idx_cases_created_by ON cases(created_by);

-- ============================================================================
-- Case Versions (for history tracking)
-- ============================================================================

CREATE TABLE case_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    snapshot JSONB NOT NULL,  -- Full case data snapshot
    changed_by UUID NOT NULL REFERENCES users(id),
    change_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_case_version UNIQUE (case_id, version)
);

CREATE INDEX idx_case_versions_case_id ON case_versions(case_id);

-- ============================================================================
-- Audit Log
-- ============================================================================

CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    table_name VARCHAR(100) NOT NULL,
    record_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT audit_action_valid CHECK (action IN ('CREATE', 'UPDATE', 'DELETE', 'SUBMIT', 'NULLIFY'))
);

CREATE INDEX idx_audit_logs_table_record ON audit_logs(table_name, record_id);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
```

### 2. E2B(R3) Section N: Message Headers

```sql
-- ============================================================================
-- SECTION N: Batch/Message Headers (Optional)
-- ============================================================================

CREATE TABLE message_headers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,

    -- N.1.1 - Batch Number
    batch_number VARCHAR(100),

    -- N.1.2 - Batch Sender Identifier
    batch_sender_identifier VARCHAR(60),

    -- Message identification
    message_type VARCHAR(50) NOT NULL,              -- ichicsr
    message_format_version VARCHAR(10) NOT NULL,     -- 2.1
    message_format_release VARCHAR(10) NOT NULL,     -- 2.0
    message_number VARCHAR(100) UNIQUE NOT NULL,
    message_sender_identifier VARCHAR(60) NOT NULL,
    message_receiver_identifier VARCHAR(60) NOT NULL,
    message_date_format VARCHAR(10) NOT NULL,        -- 204 (CCYYMMDDHHMMSS)
    message_date VARCHAR(14) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_message_per_case UNIQUE (case_id)
);

CREATE INDEX idx_message_headers_case_id ON message_headers(case_id);
CREATE INDEX idx_message_headers_number ON message_headers(message_number);
```

### 3. E2B(R3) Section C: Case Identification & Sender

```sql
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
```

### 4. E2B(R3) Section C.2.r: Primary Sources (Reporters)

```sql
-- ============================================================================
-- SECTION C.2.r: Primary Sources (Repeating)
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
```

### 5. E2B(R3) Section D: Patient Information

```sql
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
```

### 6. E2B(R3) Section E: Reaction/Event

```sql
-- ============================================================================
-- SECTION E: Reaction/Event (Repeating)
-- ============================================================================

CREATE TABLE reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,  -- For ordering multiple reactions

    -- E.i.1.1 - Reaction/Event as Reported
    primary_source_reaction VARCHAR(250) NOT NULL,

    -- E.i.1.2 - Reaction/Event Language
    reaction_language VARCHAR(2),  -- ISO 639-1 code

    -- E.i.2.1 - MedDRA Coding (LLT or PT)
    reaction_meddra_version VARCHAR(10),  -- Version of MedDRA used
    reaction_meddra_code VARCHAR(20),      -- LLT or PT code

    -- E.i.3 - Term Highlighted by Reporter
    term_highlighted BOOLEAN,

    -- E.i.3.1 - Seriousness (MANDATORY if any seriousness criteria selected)
    serious BOOLEAN,

    -- E.i.3.2 - Seriousness Criteria (at least one if serious=true)
    criteria_death BOOLEAN,
    criteria_life_threatening BOOLEAN,
    criteria_hospitalization BOOLEAN,
    criteria_disabling BOOLEAN,
    criteria_congenital_anomaly BOOLEAN,
    criteria_other_medically_important BOOLEAN,

    -- E.i.4 - Date of Start of Reaction/Event
    start_date DATE,

    -- E.i.5 - Date of End of Reaction/Event
    end_date DATE,

    -- E.i.6 - Duration of Reaction/Event
    duration_value DECIMAL(10,2),
    duration_unit VARCHAR(3),  -- 800-805 codes

    -- E.i.7 - Outcome of Reaction/Event at Time of Last Observation
    outcome VARCHAR(1) CHECK (outcome IN ('0', '1', '2', '3', '4', '5')),
    -- 0=Unknown, 1=Recovered/resolved, 2=Recovering/resolving,
    -- 3=Not recovered/not resolved, 4=Recovered/resolved with sequelae, 5=Fatal

    -- E.i.8 - Medical Confirmation by Healthcare Professional
    medical_confirmation BOOLEAN,

    -- E.i.9 - Identification of Country Where Reaction/Event Occurred
    country_code VARCHAR(2),  -- ISO 3166-1 alpha-2

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_reaction_sequence UNIQUE (case_id, sequence_number)
);

CREATE INDEX idx_reactions_case ON reactions(case_id);
CREATE INDEX idx_reactions_meddra ON reactions(reaction_meddra_code);
```

### 7. E2B(R3) Section F: Tests and Procedures

```sql
-- ============================================================================
-- SECTION F: Tests and Procedures (Repeating)
-- ============================================================================

CREATE TABLE test_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- F.r.1 - Test Date
    test_date DATE,

    -- F.r.2 - Test Name (free text or MedDRA coded)
    test_name VARCHAR(250) NOT NULL,

    -- F.r.2.1 - Test Name (MedDRA coded)
    test_meddra_version VARCHAR(10),
    test_meddra_code VARCHAR(20),

    -- F.r.3.1 - Test Result (coded)
    test_result_code VARCHAR(2),  -- E2B(R3) code list

    -- F.r.3.2 - Test Result (value/finding)
    test_result_value VARCHAR(20000),  -- Can be numeric or text

    -- F.r.3.3 - Test Result Unit
    test_result_unit VARCHAR(50),

    -- F.r.3.4 - Result Unstructured Data
    result_unstructured TEXT,

    -- F.r.4 - Normal Low Value
    normal_low_value VARCHAR(20),

    -- F.r.5 - Normal High Value
    normal_high_value VARCHAR(20),

    -- F.r.6 - Comments
    comments TEXT,

    -- F.r.7 - More Information Available
    more_info_available BOOLEAN,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_test_result_sequence UNIQUE (case_id, sequence_number)
);

CREATE INDEX idx_test_results_case ON test_results(case_id);
```

### 8. E2B(R3) Section G: Drug/Biological Information

```sql
-- ============================================================================
-- SECTION G: Drug/Biological Information (Repeating - G.k)
-- ============================================================================

CREATE TABLE drug_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,  -- k value (drug index)

    -- G.k.1 - Characterization of Drug Role (MANDATORY - E2B(R3) codes)
    drug_characterization VARCHAR(1) NOT NULL CHECK (drug_characterization IN ('1', '2', '3')),
    -- 1=Suspect, 2=Concomitant, 3=Interacting

    -- G.k.2.2 - Medicinal Product Name as Reported
    medicinal_product VARCHAR(500) NOT NULL,

    -- G.k.2.3.r - Substance/Specified Substance (repeating - handled in separate table)

    -- G.k.2.4.r - Identification of the Pharmaceutical Product (MPID)
    mpid VARCHAR(100),
    mpid_version VARCHAR(10),

    -- G.k.2.5 - PhPID (Pharmaceutical Product Identifier)
    phpid VARCHAR(100),
    phpid_version VARCHAR(10),

    -- G.k.3.1 - Obtain Drug Country
    obtain_drug_country VARCHAR(2),  -- ISO 3166-1 alpha-2

    -- G.k.3.2 - Proprietary/Brand Name
    brand_name VARCHAR(200),

    -- G.k.3.3.1 - Manufacturer Name
    manufacturer_name VARCHAR(100),

    -- G.k.3.3.2 - Manufacturer Country
    manufacturer_country VARCHAR(2),  -- ISO 3166-1 alpha-2

    -- G.k.3.4 - Batch/Lot Number
    batch_lot_number VARCHAR(200),

    -- G.k.5 - Dosage Text
    dosage_text TEXT,

    -- G.k.7 - Action(s) Taken with Drug (E2B(R3) codes)
    action_taken VARCHAR(1) CHECK (action_taken IN ('1', '2', '3', '4', '5', '6')),
    -- 1=Withdrawn, 2=Dose reduced, 3=Dose increased, 4=Dose not changed,
    -- 5=Unknown, 6=Not applicable

    -- G.k.8 - Rechallenge/Recurrence Information
    rechallenge VARCHAR(1),  -- E2B(R3) code list

    -- G.k.9 - Additional Information (handled in primary_sources table with drug FK)

    -- G.k.10 - Parent Route of Administration
    parent_route VARCHAR(3),  -- E2B(R3) code list

    -- G.k.11 - Parent Dosage Information
    parent_dosage_text TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_drug_sequence UNIQUE (case_id, sequence_number)
);

CREATE INDEX idx_drug_info_case ON drug_information(case_id);
CREATE INDEX idx_drug_info_mpid ON drug_information(mpid);

-- ============================================================================
-- G.k.2.3.r: Active Substance(s) (Repeating)
-- ============================================================================

CREATE TABLE drug_active_substances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drug_id UUID NOT NULL REFERENCES drug_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- G.k.2.3.r.1 - Substance Name
    substance_name VARCHAR(500),

    -- G.k.2.3.r.2 - Substance TermID (SUB TermID)
    substance_termid VARCHAR(100),
    substance_termid_version VARCHAR(10),

    -- G.k.2.3.r.3 - Strength (value + unit)
    strength_value DECIMAL(15,5),
    strength_unit VARCHAR(50),

    CONSTRAINT unique_substance_sequence UNIQUE (drug_id, sequence_number)
);

CREATE INDEX idx_active_substances_drug ON drug_active_substances(drug_id);

-- ============================================================================
-- G.k.4.r: Dosage and Relevant Information (Repeating)
-- ============================================================================

CREATE TABLE dosage_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drug_id UUID NOT NULL REFERENCES drug_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,  -- r value

    -- G.k.4.r.1 - Dose (value + unit)
    dose_value DECIMAL(15,5),
    dose_unit VARCHAR(50),

    -- G.k.4.r.2 - Number of Separate Dosages
    number_of_units INTEGER,

    -- G.k.4.r.3 - Dose Frequency (value + unit)
    frequency_value DECIMAL(10,2),
    frequency_unit VARCHAR(50),

    -- G.k.4.r.4 - Date/Time of First Administration
    first_administration_date DATE,
    first_administration_time TIME,

    -- G.k.4.r.5 - Date/Time of Last Administration
    last_administration_date DATE,
    last_administration_time TIME,

    -- G.k.4.r.6 - Duration of Drug Administration
    duration_value DECIMAL(10,2),
    duration_unit VARCHAR(3),  -- 800-805 codes

    -- G.k.4.r.7 - Batch/Lot Number
    batch_lot_number VARCHAR(200),

    -- G.k.4.r.8 - Dosage Text
    dosage_text TEXT,

    -- G.k.4.r.9.1 - Pharmaceutical Dose Form
    dose_form VARCHAR(200),

    -- G.k.4.r.10 - Route of Administration
    route_of_administration VARCHAR(3),  -- E2B(R3) code list

    -- G.k.4.r.11 - Parent Route of Administration
    parent_route VARCHAR(3),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_dosage_sequence UNIQUE (drug_id, sequence_number)
);

CREATE INDEX idx_dosage_info_drug ON dosage_information(drug_id);

-- ============================================================================
-- G.k.6.r: Drug Indication(s) (Repeating, MedDRA coded)
-- ============================================================================

CREATE TABLE drug_indications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drug_id UUID NOT NULL REFERENCES drug_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- G.k.6.r.1 - Indication (free text)
    indication_text VARCHAR(500),

    -- G.k.6.r.2 - Indication (MedDRA coded)
    indication_meddra_version VARCHAR(10),
    indication_meddra_code VARCHAR(20),

    CONSTRAINT unique_indication_sequence UNIQUE (drug_id, sequence_number)
);

CREATE INDEX idx_drug_indications_drug ON drug_indications(drug_id);
```

### 9. E2B(R3) Section H: Narrative and Case Summary

```sql
-- ============================================================================
-- SECTION H: Narrative and Other Information
-- ============================================================================

CREATE TABLE narrative_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,

    -- H.1 - Case Narrative Including Clinical Course, Therapeutic Measures, etc.
    case_narrative TEXT NOT NULL,  -- Free text, no length limit

    -- H.2 - Reporter's Comments
    reporter_comments TEXT,  -- Max 20000 chars

    -- H.4 - Sender's Comments
    sender_comments TEXT,  -- Max 2000 chars

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_narrative_per_case UNIQUE (case_id)
);

CREATE INDEX idx_narrative_case ON narrative_information(case_id);

-- ============================================================================
-- H.3.r: Sender's Diagnosis/Syndrome and/or Reclassification (Repeating, MedDRA)
-- ============================================================================

CREATE TABLE sender_diagnoses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    narrative_id UUID NOT NULL REFERENCES narrative_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- H.3.r.1 - Diagnosis/Syndrome (MedDRA coded)
    diagnosis_meddra_version VARCHAR(10),
    diagnosis_meddra_code VARCHAR(20),

    CONSTRAINT unique_diagnosis_sequence UNIQUE (narrative_id, sequence_number)
);

CREATE INDEX idx_sender_diagnoses ON sender_diagnoses(narrative_id);

-- ============================================================================
-- H.5.r: Case Summary and Further Information
-- ============================================================================

CREATE TABLE case_summary_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    narrative_id UUID NOT NULL REFERENCES narrative_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,

    -- H.5.r.1 - Case Summary Type
    summary_type VARCHAR(2),  -- E2B(R3) code list

    -- H.5.r.2 - Case Summary Language
    language_code VARCHAR(2),  -- ISO 639-1

    -- H.5.r.3 - Text
    summary_text TEXT,

    CONSTRAINT unique_case_summary_sequence UNIQUE (narrative_id, sequence_number)
);

CREATE INDEX idx_case_summary ON case_summary_information(narrative_id);
```

### 10. Controlled Terminology Tables

```sql
-- ============================================================================
-- Controlled Terminologies
-- ============================================================================

-- MedDRA Terms (Medical Dictionary for Regulatory Activities)
CREATE TABLE meddra_terms (
    id BIGSERIAL PRIMARY KEY,
    code VARCHAR(20) NOT NULL,
    term VARCHAR(500) NOT NULL,
    level VARCHAR(10) NOT NULL,  -- LLT, PT, HLT, HLGT, SOC
    version VARCHAR(10) NOT NULL,
    language VARCHAR(2) DEFAULT 'en',  -- ISO 639-1
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_meddra_code_version UNIQUE (code, version, language)
);

CREATE INDEX idx_meddra_code ON meddra_terms(code);
CREATE INDEX idx_meddra_term ON meddra_terms USING gin(to_tsvector('english', term));
CREATE INDEX idx_meddra_version ON meddra_terms(version);
CREATE INDEX idx_meddra_level ON meddra_terms(level);

-- WHODrug Products (WHO Drug Dictionary)
CREATE TABLE whodrug_products (
    id BIGSERIAL PRIMARY KEY,
    code VARCHAR(20) NOT NULL,
    drug_name VARCHAR(500) NOT NULL,
    atc_code VARCHAR(20),  -- Anatomical Therapeutic Chemical code
    version VARCHAR(10) NOT NULL,
    language VARCHAR(2) DEFAULT 'en',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_whodrug_code_version UNIQUE (code, version, language)
);

CREATE INDEX idx_whodrug_code ON whodrug_products(code);
CREATE INDEX idx_whodrug_name ON whodrug_products USING gin(to_tsvector('english', drug_name));
CREATE INDEX idx_whodrug_atc ON whodrug_products(atc_code);

-- ISO Country Codes
CREATE TABLE iso_countries (
    code VARCHAR(2) PRIMARY KEY,  -- ISO 3166-1 alpha-2
    name VARCHAR(200) NOT NULL,
    active BOOLEAN DEFAULT true
);

-- E2B(R3) Code Lists (enumerated values)
CREATE TABLE e2b_code_lists (
    id SERIAL PRIMARY KEY,
    list_name VARCHAR(100) NOT NULL,  -- e.g., 'report_type', 'drug_action'
    code VARCHAR(10) NOT NULL,
    display_name VARCHAR(200) NOT NULL,
    description TEXT,
    sort_order INTEGER,
    active BOOLEAN DEFAULT true,

    CONSTRAINT unique_code_per_list UNIQUE (list_name, code)
);

CREATE INDEX idx_code_lists_name ON e2b_code_lists(list_name);

-- Pre-populate common E2B(R3) code lists
INSERT INTO e2b_code_lists (list_name, code, display_name, sort_order) VALUES
-- Report Type (C.1.3)
('report_type', '1', 'Spontaneous report', 1),
('report_type', '2', 'Report from study', 2),
('report_type', '3', 'Other', 3),
('report_type', '4', 'Not available to sender', 4),

-- Sender Type (C.3.1)
('sender_type', '1', 'Pharmaceutical company', 1),
('sender_type', '2', 'Regulatory authority', 2),
('sender_type', '3', 'Health professional', 3),
('sender_type', '4', 'Regional pharmacovigilance center', 4),
('sender_type', '5', 'WHO collaborating center for international drug monitoring', 5),
('sender_type', '6', 'Other', 6),

-- Qualification (C.2.r.4)
('qualification', '1', 'Physician', 1),
('qualification', '2', 'Pharmacist', 2),
('qualification', '3', 'Other health professional', 3),
('qualification', '4', 'Lawyer', 4),
('qualification', '5', 'Consumer or other non health professional', 5),

-- Sex (D.5)
('sex', '0', 'Unknown', 0),
('sex', '1', 'Male', 1),
('sex', '2', 'Female', 2),

-- Age Group (D.2.3)
('age_group', '1', 'Neonate', 1),
('age_group', '2', 'Infant', 2),
('age_group', '3', 'Child', 3),
('age_group', '4', 'Adolescent', 4),
('age_group', '5', 'Adult', 5),
('age_group', '6', 'Elderly', 6),

-- Age Unit (D.2.2)
('age_unit', '800', 'Decade', 1),
('age_unit', '801', 'Year', 2),
('age_unit', '802', 'Month', 3),
('age_unit', '803', 'Week', 4),
('age_unit', '804', 'Day', 5),
('age_unit', '805', 'Hour', 6),

-- Reaction Outcome (E.i.7)
('reaction_outcome', '0', 'Unknown', 0),
('reaction_outcome', '1', 'Recovered/resolved', 1),
('reaction_outcome', '2', 'Recovering/resolving', 2),
('reaction_outcome', '3', 'Not recovered/not resolved', 3),
('reaction_outcome', '4', 'Recovered/resolved with sequelae', 4),
('reaction_outcome', '5', 'Fatal', 5),

-- Drug Characterization (G.k.1)
('drug_characterization', '1', 'Suspect', 1),
('drug_characterization', '2', 'Concomitant', 2),
('drug_characterization', '3', 'Interacting', 3),

-- Drug Action Taken (G.k.7)
('drug_action', '1', 'Drug withdrawn', 1),
('drug_action', '2', 'Dose reduced', 2),
('drug_action', '3', 'Dose increased', 3),
('drug_action', '4', 'Dose not changed', 4),
('drug_action', '5', 'Unknown', 5),
('drug_action', '6', 'Not applicable', 6);
```

### 11. Database Functions and Triggers

```sql
-- ============================================================================
-- Utility Functions
-- ============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to all tables with updated_at
CREATE TRIGGER update_cases_updated_at BEFORE UPDATE ON cases
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_patient_info_updated_at BEFORE UPDATE ON patient_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_reactions_updated_at BEFORE UPDATE ON reactions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_drug_info_updated_at BEFORE UPDATE ON drug_information
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add similar triggers for all tables with updated_at column

-- ============================================================================
-- Audit Trail Trigger
-- ============================================================================

CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, new_values)
        VALUES (TG_TABLE_NAME, NEW.id, 'CREATE', current_setting('app.current_user_id', true)::UUID, to_jsonb(NEW));
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values, new_values)
        VALUES (TG_TABLE_NAME, NEW.id, 'UPDATE', current_setting('app.current_user_id', true)::UUID, to_jsonb(OLD), to_jsonb(NEW));
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values)
        VALUES (TG_TABLE_NAME, OLD.id, 'DELETE', current_setting('app.current_user_id', true)::UUID, to_jsonb(OLD));
        RETURN OLD;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Apply audit triggers to critical tables
CREATE TRIGGER audit_cases AFTER INSERT OR UPDATE OR DELETE ON cases
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_patient_info AFTER INSERT OR UPDATE OR DELETE ON patient_information
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- Add similar audit triggers for all E2B(R3) data tables
```

### Critical Schema Requirements

1. **Null Flavor Support**: E2B(R3) requires specific null flavor codes (ASKU, NASK, MSK, etc.)
   - Fields that accept null flavors are marked with VARCHAR allowing special codes
   - Implement validation at application level for null flavor codes

2. **Cardinality Constraints**: Enforced via:
   - Database foreign key constraints
   - Unique constraints for 1-to-1 relationships
   - Application-level validation for min/max occurrence rules

3. **Controlled Terminologies**:
   - MedDRA terms table for reactions, indications, medical history
   - WHODrug table for drug products
   - ISO country codes table
   - E2B code lists table for all enumerated values

4. **Audit Trail**: Comprehensive logging via triggers
   - All INSERT/UPDATE/DELETE operations logged
   - User ID tracking (set via `SET app.current_user_id = '<uuid>'`)
   - Old and new values captured as JSONB

5. **Version History**: Supported via case_versions table
   - Full case snapshot stored as JSONB on each save
   - Version number tracked in cases table
   - Supports follow-up reports and amendments

## E2B(R3) Compliance Validation

### Using the e2br3-compliance-validator Agent

**CRITICAL**: Before implementing or modifying the database schema, ALWAYS use the specialized E2B(R3) validation agent.

#### When to Use

1. **Initial Schema Design**: Before creating any migration files
2. **Schema Modifications**: After any changes to case-related tables
3. **Before Production Deploy**: Final validation before going live
4. **Mapping Legacy Data**: When migrating from old systems

#### Agent Capabilities

The agent will:
1. Read E2B(R3) technical documentation from `/Users/hyundonghoon/Documents`
2. Analyze your schema against ALL mandatory/optional data elements
3. Verify cardinality constraints (min/max occurrences)
4. Check data types and lengths
5. Validate controlled vocabulary implementations
6. Generate compliance report with specific gaps
7. Provide remediation SQL with migration strategy
8. Include rollback plan

#### Documentation Access

The agent has access to complete E2B(R3) documentation:

**Location**: `/Users/hyundonghoon/Documents`

1. **XSD Schema Files** (`4_ICH_ICSR_Schema_Files/`)
   - Core schemas: datatypes-base.xsd, datatypes.xsd, infrastructureRoot.xsd
   - Multicache schemas: 200+ XSD files for all E2B(R3) components
   - Use for XML validation implementation

2. **Example Instances** (`6_Example Instances/`)
   - 5 official ICH test cases with XML and Excel documentation
   - Use for testing XML parser and generator
   - Integration test data

3. **Technical Information** (`8_Technical Information/`)
   - `8_Technical Information_v1_02.pdf`: Complete implementation guide
   - `8_Reference_v1_10.xlsx`: Data element reference spreadsheet
   - Use for database schema design

#### Example Validation Workflow

```bash
# 1. Design initial schema (SQL or ORM migrations)

# 2. Use validator agent (in Claude Code)
# Agent will:
# - Read schema from your migration files
# - Compare against E2B(R3) requirements
# - Generate compliance report

# 3. Review compliance report
# - Check coverage percentage
# - Identify missing mandatory fields
# - Review cardinality issues

# 4. Implement remediation SQL
# - Agent provides specific ALTER TABLE statements
# - Includes data type corrections
# - Adds missing constraints

# 5. Re-validate after changes

# 6. Proceed with implementation only when compliant
```

## REST API Specification

### Authentication Endpoints

```
POST   /api/auth/register
  Request:  { email, password, organization_id }
  Response: { user_id, message }

POST   /api/auth/login
  Request:  { email, password }
  Response: Set-Cookie: jwt=...; HttpOnly
            { user: { id, email, role, organization_id } }

POST   /api/auth/refresh
  Request:  Cookie: jwt=...
  Response: Set-Cookie: jwt=...; HttpOnly
            { user: { id, email, role, organization_id } }

POST   /api/auth/logout
  Request:  Cookie: jwt=...
  Response: Clear-Cookie: jwt
            { message: "Logged out" }
```

### Case Management Endpoints

```
GET    /api/cases
  Query:   ?page=1&limit=20&status=draft&search=...
  Response: { cases: [], total: number, page: number }

POST   /api/cases
  Request:  { /* case data matching E2B(R3) schema */ }
  Response: { case_id: UUID, message: "Created" }

GET    /api/cases/:id
  Response: { /* complete case data */ }

PUT    /api/cases/:id
  Request:  { /* updated case data */ }
  Response: { case_id: UUID, message: "Updated" }

DELETE /api/cases/:id
  Response: { message: "Deleted" }

PATCH  /api/cases/:id/status
  Request:  { status: "draft" | "submitted" | "validated" }
  Response: { case_id: UUID, status: string }
```

### XML Processing Endpoints

```
GET    /api/xml/export/:case_id
  Response: Content-Type: application/xml
            E2B(R3) XML document (XSD validated)

POST   /api/xml/import
  Request:  multipart/form-data with XML file
  Response: { case_id: UUID, validation_report: { ... } }

POST   /api/xml/validate
  Request:  multipart/form-data with XML file
  Response: { valid: boolean, errors: [] }
```

### Terminology Endpoints

```
GET    /api/terminology/meddra
  Query:   ?search=headache&level=PT&limit=10
  Response: { terms: [{ code, term, level }] }

GET    /api/terminology/whodrug
  Query:   ?search=aspirin&limit=10
  Response: { terms: [{ code, term, atc_code }] }

GET    /api/terminology/countries
  Response: { countries: [{ code, name }] }
```

### Request/Response Standards

**Success Response** (200, 201):
```json
{
  "data": { /* response data */ },
  "status": "success"
}
```

**Error Response** (400, 401, 403, 404, 500):
```json
{
  "error": {
    "message": "Human-readable error message",
    "code": "ERROR_CODE",
    "details": { /* optional validation errors */ }
  },
  "status": "error"
}
```

**Validation Error** (422):
```json
{
  "error": {
    "message": "Validation failed",
    "code": "VALIDATION_ERROR",
    "details": {
      "fields": {
        "patient_age": ["Required field missing"],
        "reaction_meddra_code": ["Invalid MedDRA code"]
      }
    }
  },
  "status": "error"
}
```

## XML Processing Implementation

### Requirements

1. **XSD Validation**
   - Validate all generated XML against official ICH XSD schemas
   - Schemas located in `/Users/hyundonghoon/Documents/4_ICH_ICSR_Schema_Files/`
   - Must pass validation before sending to regulatory authorities

2. **Controlled Terminology**
   - MedDRA codes for reactions and indications
   - WHODrug codes for products
   - ISO country codes, date formats
   - E2B-specific code lists

3. **Message Headers**
   - Implement proper ICH message header structure
   - Include sender/receiver identifiers
   - Message numbering and version tracking
   - Support regional requirements (FDA, EMA, PMDA)

4. **Null Flavor Handling**
   - Implement proper null flavor codes when data unavailable
   - ASKU (asked but unknown), NASK (not asked), MSK (masked), etc.
   - Required for E2B(R3) compliance

### Example XML Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ichicsr lang="en" xmlns="http://www.ich.org/icsr">
  <ichicsrmessageheader>
    <messagetype>ichicsr</messagetype>
    <messageformatversion>2.1</messageformatversion>
    <messageformatrelease>2.0</messageformatrelease>
    <messagenumb>US-COMPANY-2024-001234</messagenumb>
    <messagesenderidentifier>US-FDA-COMPANY</messagesenderidentifier>
    <messagereceiveridentifier>US-FDA</messagereceiveridentifier>
    <messagedateformat>204</messagedateformat>
    <messagedate>20240101120000</messagedate>
  </ichicsrmessageheader>
  <safetyreport>
    <!-- Complete E2B(R3) structure with all sections A-H -->
  </safetyreport>
</ichicsr>
```

### Testing with Official Examples

Use the 5 official ICH test cases in `/Users/hyundonghoon/Documents/6_Example Instances/`:

1. **1-1_ExampleCase_literature_initial_v1_0.xml**
   - Test literature-based case import/export

2. **1-2_ExampleCase_nullification_v1_0.xml**
   - Test nullification workflow

3. **2-1_ExampleCase_newNum_initial_v1_0.xml**
   - Test new case number assignment

4. **2-2_ExampleCase_newNum_F-up_v1_0.xml**
   - Test follow-up report handling

5. **3_ExampleCase_Clinical_trial_v1_0.xml**
   - Test clinical trial cases

**Implementation Checklist**:
- [ ] XML parser can read all 5 examples without errors
- [ ] Generated XML matches ICH structure
- [ ] XSD validation passes for all generated XML
- [ ] Round-trip test: import XML → export XML → compare
- [ ] All controlled terminologies validated

## Security Requirements

### Authentication

1. **JWT Implementation**
   - Use HS256 or RS256 algorithm
   - Short-lived access tokens (15 minutes)
   - Long-lived refresh tokens (7 days)
   - Store refresh tokens in httpOnly cookies

2. **Password Security**
   - Use bcrypt with cost factor 12+
   - Minimum password requirements (length, complexity)
   - Password reset with time-limited tokens
   - Account lockout after failed attempts

3. **Session Management**
   - Track active sessions in database
   - Support session invalidation (logout)
   - Automatic cleanup of expired sessions

### Authorization

1. **Role-Based Access Control (RBAC)**
   ```
   Roles:
   - Admin: Full system access
   - Manager: Manage cases within organization
   - User: Create/view own cases
   - Viewer: Read-only access
   ```

2. **Organization Isolation**
   - All queries filtered by user's organization
   - Prevent cross-organization data access
   - Organization-level permissions

3. **API Security**
   - CSRF protection for state-changing operations
   - Rate limiting on all endpoints
   - Input validation and sanitization
   - SQL injection prevention (use parameterized queries)

### Data Security

1. **Encryption**
   - TLS 1.3 for all connections
   - Encrypt sensitive fields at rest (optional)
   - Secure key management

2. **Audit Trail**
   - Log all data access and modifications
   - Store user, timestamp, action, old/new values
   - Immutable audit log (append-only)

3. **Data Retention**
   - Define retention policies per regulatory requirements
   - Implement soft deletes (mark deleted, don't remove)
   - Support data export for compliance

## Backend Production Roadmap

See `PRODUCTION-PLAN.md` for the complete roadmap with dependencies.

### Phase 1: Core Infrastructure

1. **Project Setup**
   - Initialize Rust project with cargo
   - Choose web framework (Axum or Actix-web)
   - Setup PostgreSQL connection pool
   - Configure environment variables

2. **Database Layer**
   - Design E2B(R3) schema (use validator agent)
   - Implement migrations (SQLx or Diesel)
   - Create database models/entities
   - Setup connection pooling

3. **Authentication System**
   - Implement JWT generation/validation
   - Create user registration/login endpoints
   - Add password hashing with bcrypt
   - Setup httpOnly cookie handling

### Phase 2: E2B(R3) Implementation

1. **Complete Data Model**
   - Implement ALL E2B(R3) sections (A-H)
   - Add MedDRA/WHODrug lookup tables
   - Implement cardinality constraints
   - Add controlled terminology validation

2. **Case Management API**
   - CRUD endpoints for cases
   - Validation logic for all fields
   - Business rule enforcement
   - Status workflow management

3. **XML Processing**
   - Install XML libraries (quick-xml)
   - Implement XSD validation
   - Create XML generator for E2B(R3)
   - Create XML parser with validation
   - Test against ICH examples

### Phase 3: Production Readiness

1. **Testing**
   - Unit tests for all business logic
   - Integration tests for API endpoints
   - Test against all 5 ICH examples
   - Load testing for performance

2. **Monitoring & Logging**
   - Structured logging (tracing crate)
   - Error tracking and alerting
   - Performance monitoring
   - Database query optimization

3. **Deployment**
   - Containerization (Docker)
   - CI/CD pipeline setup
   - Database backup strategy
   - Rollback procedures

## Critical Backend Notes

1. **E2B(R3) Compliance**: Always validate schema changes with the e2br3-compliance-validator agent
2. **XSD Validation**: Never generate XML without validating against official XSD schemas
3. **Controlled Terminologies**: Keep MedDRA/WHODrug tables updated with latest versions
4. **Audit Trail**: Every data change must be logged for regulatory compliance
5. **Data Integrity**: Use database transactions for multi-table operations
6. **Regional Requirements**: Support different regulatory requirements (FDA vs EMA vs PMDA)
7. **Versioning**: Implement proper case versioning for follow-ups and amendments

## Related Documentation

- **Frontend**: See `CLAUDE.md` in the E2BR3-frontend repository
- **Production Plan**: See `PRODUCTION-PLAN.md` for complete roadmap
- **E2B(R3) Spec**: `/Users/hyundonghoon/Documents/8_Technical Information/8_Technical Information_v1_02.pdf`
