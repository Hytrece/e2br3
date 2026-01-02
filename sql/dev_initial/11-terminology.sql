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