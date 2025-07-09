-- Drop schema for Metis document storage system

-- Drop triggers first
DROP TRIGGER IF EXISTS documents_ad;
DROP TRIGGER IF EXISTS documents_au;
DROP TRIGGER IF EXISTS documents_ai;

-- Drop indexes
DROP INDEX IF EXISTS idx_tags_tag;
DROP INDEX IF EXISTS idx_relationships_parent;
DROP INDEX IF EXISTS idx_relationships_child;
DROP INDEX IF EXISTS idx_documents_updated;
DROP INDEX IF EXISTS idx_documents_type;
DROP INDEX IF EXISTS idx_documents_id;

-- Drop FTS table
DROP TABLE IF EXISTS document_search;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS document_tags;
DROP TABLE IF EXISTS document_relationships;
DROP TABLE IF EXISTS documents;