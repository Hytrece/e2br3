CREATE TABLE IF NOT EXISTS mfds_product_substances (
    id BIGSERIAL PRIMARY KEY,
    audit_id UUID NOT NULL DEFAULT gen_random_uuid(),
    item_seq VARCHAR(10) NOT NULL,
    substance_code VARCHAR(40) NOT NULL,
    substance_name_kr TEXT NOT NULL,
    substance_name_en TEXT,
    quantity TEXT,
    unit TEXT,
    component_content TEXT,
    material_sequence TEXT NOT NULL DEFAULT '',
    total_amount_sequence TEXT NOT NULL DEFAULT '',
    version VARCHAR(40) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT mfds_product_substances_unique
        UNIQUE (item_seq, substance_code, material_sequence, total_amount_sequence, version),
    CONSTRAINT mfds_product_substances_product_fk
        FOREIGN KEY (item_seq, version) REFERENCES mfds_products (item_seq, version) ON DELETE CASCADE,
    CONSTRAINT mfds_product_substances_audit_id_unique UNIQUE (audit_id)
);

CREATE INDEX IF NOT EXISTS idx_mfds_product_substances_lookup
    ON mfds_product_substances (item_seq, version) WHERE active = true;

GRANT SELECT, INSERT, UPDATE, DELETE ON mfds_product_substances TO e2br3_app_role;
GRANT USAGE, SELECT ON SEQUENCE mfds_product_substances_id_seq TO e2br3_app_role;

ALTER TABLE mfds_product_substances ENABLE ROW LEVEL SECURITY;
ALTER TABLE mfds_product_substances FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS mfds_product_substances_read ON mfds_product_substances;
CREATE POLICY mfds_product_substances_read ON mfds_product_substances
    FOR SELECT TO e2br3_app_role
    USING (active = true OR is_current_user_admin());
DROP POLICY IF EXISTS mfds_product_substances_insert ON mfds_product_substances;
CREATE POLICY mfds_product_substances_insert ON mfds_product_substances
    FOR INSERT TO e2br3_app_role WITH CHECK (is_current_user_admin());
DROP POLICY IF EXISTS mfds_product_substances_update ON mfds_product_substances;
CREATE POLICY mfds_product_substances_update ON mfds_product_substances
    FOR UPDATE TO e2br3_app_role
    USING (is_current_user_admin()) WITH CHECK (is_current_user_admin());
DROP POLICY IF EXISTS mfds_product_substances_delete ON mfds_product_substances;
CREATE POLICY mfds_product_substances_delete ON mfds_product_substances
    FOR DELETE TO e2br3_app_role USING (is_current_user_admin());

DROP TRIGGER IF EXISTS audit_mfds_product_substances ON mfds_product_substances;
CREATE TRIGGER audit_mfds_product_substances
    AFTER INSERT OR UPDATE OR DELETE ON mfds_product_substances
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function_with_audit_id();
