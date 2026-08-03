UPDATE reporter_presaves
SET primary_source_regulatory = NULL
WHERE primary_source_regulatory IS NOT NULL
  AND primary_source_regulatory <> '1';

UPDATE product_presaves
SET investigational_product_blinded = NULL
WHERE investigational_product_blinded = FALSE;
