DROP INDEX IF EXISTS idx_documents_parent_id;
ALTER TABLE documents DROP COLUMN parent_id;
