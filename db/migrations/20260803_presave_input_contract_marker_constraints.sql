ALTER TABLE reporter_presaves
ADD CONSTRAINT reporter_presaves_primary_source_regulatory_input_contract
CHECK (primary_source_regulatory IS NULL OR primary_source_regulatory = '1');

ALTER TABLE product_presaves
ADD CONSTRAINT product_presaves_investigational_product_blinded_input_contract
CHECK (investigational_product_blinded IS NULL OR investigational_product_blinded = TRUE);
