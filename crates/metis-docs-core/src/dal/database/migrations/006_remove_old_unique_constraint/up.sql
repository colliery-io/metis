-- Change unique constraint from (document_type, id) to (document_type, short_code)
-- SQLite doesn't support DROP CONSTRAINT directly, so we need to recreate the table

-- Create temp table with correct unique constraint on (document_type, short_code)
CREATE TABLE documents_temp (
    filepath TEXT PRIMARY KEY NOT NULL,
    id TEXT NOT NULL,
    title TEXT NOT NULL,
    document_type TEXT NOT NULL,
    created_at REAL NOT NULL,
    updated_at REAL NOT NULL,
    archived BOOLEAN NOT NULL DEFAULT FALSE,
    exit_criteria_met BOOLEAN NOT NULL DEFAULT FALSE,
    file_hash TEXT NOT NULL,
    frontmatter_json TEXT NOT NULL,
    content TEXT,
    phase TEXT NOT NULL DEFAULT 'draft',
    strategy_id TEXT,
    initiative_id TEXT,
    short_code TEXT NOT NULL DEFAULT 'NULL',
    UNIQUE(document_type, short_code)
);

-- Copy ALL data from old table to temp table
INSERT INTO documents_temp SELECT * FROM documents;

-- Drop old table
DROP TABLE documents;

-- Rename temp table to final name
ALTER TABLE documents_temp RENAME TO documents;

-- Recreate indexes (keeping the same structure)
CREATE INDEX idx_documents_id ON documents(id);
CREATE INDEX idx_documents_type ON documents(document_type);
CREATE INDEX idx_documents_updated ON documents(updated_at);
CREATE INDEX idx_documents_phase ON documents(phase);
CREATE INDEX idx_documents_strategy_id ON documents(strategy_id);
CREATE INDEX idx_documents_initiative_id ON documents(initiative_id);
CREATE INDEX idx_documents_lineage ON documents(strategy_id, initiative_id);
CREATE UNIQUE INDEX idx_documents_short_code ON documents(short_code);

-- Recreate FTS triggers
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

CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
    DELETE FROM document_search WHERE document_filepath = old.filepath;
END;