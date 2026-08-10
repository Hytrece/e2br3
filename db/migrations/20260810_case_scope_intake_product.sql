CREATE OR REPLACE FUNCTION case_scope_identifiers(target_case_id UUID)
RETURNS TABLE(scope_kind TEXT, identifier TEXT)
LANGUAGE SQL
STABLE
AS $$
    SELECT 'sender', source_sender_presave_id::text FROM sender_information
     WHERE case_id = target_case_id AND source_sender_presave_id IS NOT NULL
    UNION SELECT 'product', source_product_presave_id::text FROM drug_information
     WHERE case_id = target_case_id AND source_product_presave_id IS NOT NULL
    UNION SELECT 'study', source_study_presave_id::text FROM study_information
     WHERE case_id = target_case_id AND source_study_presave_id IS NOT NULL
    UNION SELECT 'sender', product.sender_presave_id::text FROM drug_information drug
      JOIN product_presaves product ON product.id = drug.source_product_presave_id
     WHERE drug.case_id = target_case_id AND product.sender_presave_id IS NOT NULL
    UNION SELECT 'product', study.product_presave_id::text FROM study_information case_study
      JOIN study_presaves study ON study.id = case_study.source_study_presave_id
     WHERE case_study.case_id = target_case_id AND study.product_presave_id IS NOT NULL
    UNION SELECT 'sender', product.sender_presave_id::text FROM study_information case_study
      JOIN study_presaves study ON study.id = case_study.source_study_presave_id
      JOIN product_presaves product ON product.id = study.product_presave_id
     WHERE case_study.case_id = target_case_id AND product.sender_presave_id IS NOT NULL
    UNION SELECT 'product', product.id::text FROM cases case_row
      JOIN product_presaves product ON product.organization_id = case_row.organization_id
       AND product.product_id = case_row.dg_prd_key AND product.deleted = false
     WHERE case_row.id = target_case_id
    UNION SELECT 'sender', product.sender_presave_id::text FROM cases case_row
      JOIN product_presaves product ON product.organization_id = case_row.organization_id
       AND product.product_id = case_row.dg_prd_key AND product.deleted = false
     WHERE case_row.id = target_case_id AND product.sender_presave_id IS NOT NULL;
$$;
