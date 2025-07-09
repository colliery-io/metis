-- Rollback phase column addition

-- Drop phase index
DROP INDEX IF EXISTS idx_documents_phase;

-- Remove phase column from documents table
ALTER TABLE documents DROP COLUMN phase;

-- Recreate original triggers without phase
DROP TRIGGER documents_ai;
DROP TRIGGER documents_au;

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