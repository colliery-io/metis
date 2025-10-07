-- Add short_code column to documents table
ALTER TABLE documents ADD COLUMN short_code TEXT NOT NULL DEFAULT 'NULL';

-- Create unique index on short_code for fast lookups and uniqueness constraint
CREATE UNIQUE INDEX idx_documents_short_code ON documents(short_code);