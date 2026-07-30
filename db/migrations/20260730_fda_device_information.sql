-- FDA.G.k.12.r is a repeating device group. Its .2.r, .3.r and .11.r
-- elements repeat independently within each device.

CREATE TABLE IF NOT EXISTS fda_device_information (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drug_id UUID NOT NULL REFERENCES drug_information(id) ON DELETE CASCADE,
    sequence_number INTEGER NOT NULL,
    malfunction BOOLEAN,
    device_brand_name VARCHAR(80),
    device_brand_name_null_flavor VARCHAR(2) CHECK (device_brand_name_null_flavor IN ('NI')),
    common_device_name VARCHAR(80),
    common_device_name_null_flavor VARCHAR(2) CHECK (common_device_name_null_flavor IN ('NI')),
    device_product_code VARCHAR(10),
    manufacturer_name VARCHAR(100),
    manufacturer_address VARCHAR(100),
    manufacturer_city VARCHAR(35),
    manufacturer_state VARCHAR(40),
    manufacturer_country VARCHAR(2),
    device_usage VARCHAR(1) CHECK (device_usage IN ('1', '2', '3')),
    device_lot_number VARCHAR(100),
    operator_of_device VARCHAR(1) CHECK (operator_of_device IN ('1', '2', '3')),
    deleted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by UUID REFERENCES users(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_fda_device_information_drug
    ON fda_device_information(drug_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_fda_device_information_active_sequence_unique
    ON fda_device_information(drug_id, sequence_number)
    WHERE deleted = false;

CREATE TABLE IF NOT EXISTS fda_device_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES fda_device_information(id) ON DELETE CASCADE,
    element VARCHAR(20) NOT NULL CHECK (element IN ('follow_up_type', 'device_problem', 'remedial_action')),
    sequence_number INTEGER NOT NULL,
    value_code VARCHAR(7) NOT NULL,
    CHECK (
        (element = 'follow_up_type' AND value_code IN ('1', '2', '3', '4')) OR
        (element = 'device_problem' AND char_length(value_code) BETWEEN 1 AND 7) OR
        (element = 'remedial_action' AND value_code IN ('1', '2', '3', '4', '5', '6', '7', '8', '9'))
    ),
    deleted BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by UUID REFERENCES users(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_fda_device_codes_device
    ON fda_device_codes(device_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_fda_device_codes_active_sequence_unique
    ON fda_device_codes(device_id, element, sequence_number)
    WHERE deleted = false;

ALTER TABLE drug_information DROP COLUMN IF EXISTS fda_device_info_json;
