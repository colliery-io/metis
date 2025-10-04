-- Add lineage tracking columns to documents table
ALTER TABLE documents ADD COLUMN strategy_id TEXT;
ALTER TABLE documents ADD COLUMN initiative_id TEXT;

-- Create indexes for efficient lineage queries
CREATE INDEX idx_documents_strategy_id ON documents(strategy_id);
CREATE INDEX idx_documents_initiative_id ON documents(initiative_id);

-- Create compound index for strategy+initiative queries
CREATE INDEX idx_documents_lineage ON documents(strategy_id, initiative_id);