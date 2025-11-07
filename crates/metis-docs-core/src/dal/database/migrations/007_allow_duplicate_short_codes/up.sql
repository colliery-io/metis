-- Remove UNIQUE constraint on short_code to allow temporary duplicates during branch merges
-- This enables multi-developer workflows where sync will lazily resolve collisions

-- Drop the unique index on short_code
DROP INDEX IF EXISTS idx_documents_short_code;

-- Create non-unique index for performance on short_code lookups
CREATE INDEX idx_documents_short_code ON documents(short_code);
