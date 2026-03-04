ALTER TABLE documents ADD COLUMN parent_id TEXT;
CREATE INDEX idx_documents_parent_id ON documents(parent_id);
