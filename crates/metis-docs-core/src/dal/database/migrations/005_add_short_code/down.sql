-- Remove short_code column and its index
DROP INDEX IF EXISTS idx_documents_short_code;
ALTER TABLE documents DROP COLUMN short_code;