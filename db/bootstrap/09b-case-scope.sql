CREATE OR REPLACE FUNCTION case_scope_identifiers(target_case_id UUID)
RETURNS TABLE(scope_kind TEXT, identifier TEXT)
LANGUAGE SQL
STABLE
AS $$
    SELECT 'sender', source_sender_presave_id::text
      FROM sender_information
     WHERE case_id = target_case_id
       AND source_sender_presave_id IS NOT NULL
    UNION
    SELECT 'product', source_product_presave_id::text
      FROM drug_information
     WHERE case_id = target_case_id
       AND source_product_presave_id IS NOT NULL
    UNION
    SELECT 'study', source_study_presave_id::text
      FROM study_information
     WHERE case_id = target_case_id
       AND source_study_presave_id IS NOT NULL
    UNION
    SELECT 'sender', product.sender_presave_id::text
      FROM drug_information drug
      JOIN product_presaves product ON product.id = drug.source_product_presave_id
     WHERE drug.case_id = target_case_id
       AND product.sender_presave_id IS NOT NULL
    UNION
    SELECT 'product', study.product_presave_id::text
      FROM study_information case_study
      JOIN study_presaves study ON study.id = case_study.source_study_presave_id
     WHERE case_study.case_id = target_case_id
       AND study.product_presave_id IS NOT NULL
    UNION
    SELECT 'sender', product.sender_presave_id::text
      FROM study_information case_study
      JOIN study_presaves study ON study.id = case_study.source_study_presave_id
      JOIN product_presaves product ON product.id = study.product_presave_id
     WHERE case_study.case_id = target_case_id
       AND product.sender_presave_id IS NOT NULL
    UNION
    SELECT 'product', product.id::text
      FROM cases case_row
      JOIN product_presaves product
        ON product.organization_id = case_row.organization_id
       AND product.product_id = case_row.dg_prd_key
       AND product.deleted = false
     WHERE case_row.id = target_case_id
    UNION
    SELECT 'sender', product.sender_presave_id::text
      FROM cases case_row
      JOIN product_presaves product
        ON product.organization_id = case_row.organization_id
       AND product.product_id = case_row.dg_prd_key
       AND product.deleted = false
     WHERE case_row.id = target_case_id
       AND product.sender_presave_id IS NOT NULL;
$$;

CREATE OR REPLACE FUNCTION validate_scope_assignment(
    target_organization_id UUID,
    sender_ids TEXT[],
    product_ids TEXT[],
    study_ids TEXT[]
)
RETURNS TEXT
LANGUAGE plpgsql
STABLE
AS $$
BEGIN
    IF EXISTS (
        SELECT 1
          FROM unnest(sender_ids) requested(id)
         WHERE NOT EXISTS (
             SELECT 1
               FROM sender_presaves sender
              WHERE sender.organization_id = target_organization_id
                AND sender.deleted = false
                AND sender.id::text = requested.id
         )
    ) THEN
        RETURN 'sender_not_found';
    END IF;

    IF EXISTS (
        SELECT 1
          FROM unnest(product_ids) requested(id)
         WHERE NOT EXISTS (
             SELECT 1
               FROM product_presaves product
              WHERE product.organization_id = target_organization_id
                AND product.deleted = false
                AND product.id::text = requested.id
                AND (
                    cardinality(sender_ids) = 0
                    OR product.sender_presave_id::text = ANY(sender_ids)
                )
         )
    ) THEN
        RETURN 'product_sender_mismatch';
    END IF;

    IF EXISTS (
        SELECT 1
          FROM unnest(study_ids) requested(id)
         WHERE NOT EXISTS (
             SELECT 1
               FROM study_presaves study
              WHERE study.organization_id = target_organization_id
                AND study.deleted = false
                AND study.id::text = requested.id
                AND (
                    cardinality(product_ids) = 0
                    OR study.product_presave_id::text = ANY(product_ids)
                )
         )
    ) THEN
        RETURN 'study_product_mismatch';
    END IF;

    IF EXISTS (
        SELECT 1
          FROM unnest(study_ids) requested(id)
         WHERE NOT EXISTS (
             SELECT 1
               FROM study_presaves study
               JOIN product_presaves product
                 ON product.id = study.product_presave_id
                AND product.organization_id = target_organization_id
                AND product.deleted = false
              WHERE study.organization_id = target_organization_id
                AND study.deleted = false
                AND study.id::text = requested.id
                AND product.sender_presave_id::text = ANY(sender_ids)
         )
    ) AND cardinality(sender_ids) > 0 THEN
        RETURN 'study_sender_mismatch';
    END IF;

    RETURN NULL;
END;
$$;
