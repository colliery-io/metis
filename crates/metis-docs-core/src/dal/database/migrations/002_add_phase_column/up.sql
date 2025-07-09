-- Add phase column to documents table for better filtering
-- Phase tracking for document lifecycle management

-- Add phase column to documents table
ALTER TABLE documents ADD COLUMN phase TEXT NOT NULL DEFAULT 'draft';

-- Create index for phase-based queries
CREATE INDEX idx_documents_phase ON documents(phase);

-- Update FTS triggers to include phase in search index
DROP TRIGGER documents_ai;
DROP TRIGGER documents_au;

-- Recreate triggers with phase included
CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
    INSERT INTO document_search(document_filepath, content, title, document_type)
    VALUES (new.filepath, new.content, new.title, new.document_type);
END;

CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
    UPDATE document_search 
    SET content = new.content,
        title = new.title,
        document_type = new.document_type
    WHERE document_filepath = new.filepath;
END;