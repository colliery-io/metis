-- Drop indexes first
DROP INDEX IF EXISTS idx_documents_lineage;
DROP INDEX IF EXISTS idx_documents_initiative_id;
DROP INDEX IF EXISTS idx_documents_strategy_id;

-- Drop lineage tracking columns
ALTER TABLE documents DROP COLUMN initiative_id;
ALTER TABLE documents DROP COLUMN strategy_id;