-- Restore UNIQUE constraint on short_code (rollback migration)

-- Drop the non-unique index
DROP INDEX IF EXISTS idx_documents_short_code;

-- Recreate unique index on short_code
CREATE UNIQUE INDEX idx_documents_short_code ON documents(short_code);
