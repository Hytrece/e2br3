-- MPID permits 1,000 characters, which can exceed PostgreSQL's btree entry
-- size for multibyte text. No application query relies on this index.
DROP INDEX IF EXISTS idx_drug_info_mpid;
