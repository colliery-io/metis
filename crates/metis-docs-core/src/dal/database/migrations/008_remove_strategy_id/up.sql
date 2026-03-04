-- Remove strategy_id column and delete strategy documents
-- SQLite doesn't support DROP COLUMN, so we recreate the table

-- Delete any strategy documents first
DELETE FROM document_tags WHERE document_filepath IN (
    SELECT filepath FROM documents WHERE document_type = 'strategy'
);
DELETE FROM document_relationships WHERE child_filepath IN (
    SELECT filepath FROM documents WHERE document_type = 'strategy'
) OR parent_filepath IN (
    SELECT filepath FROM documents WHERE document_type = 'strategy'
);
DELETE FROM document_search WHERE document_filepath IN (
    SELECT filepath FROM documents WHERE document_type = 'strategy'
);
DELETE FROM documents WHERE document_type = 'strategy';

-- Drop strategy-related indexes
DROP INDEX IF EXISTS idx_documents_strategy_id;
DROP INDEX IF EXISTS idx_documents_lineage;

-- Recreate documents table without strategy_id
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
    initiative_id TEXT,
    short_code TEXT NOT NULL DEFAULT 'NULL'
);

-- Copy data (excluding strategy_id column)
INSERT INTO documents_temp (
    filepath, id, title, document_type, created_at, updated_at,
    archived, exit_criteria_met, file_hash, frontmatter_json,
    content, phase, initiative_id, short_code
)
SELECT
    filepath, id, title, document_type, created_at, updated_at,
    archived, exit_criteria_met, file_hash, frontmatter_json,
    content, phase, initiative_id, short_code
FROM documents;

-- Drop old table and rename
DROP TABLE documents;
ALTER TABLE documents_temp RENAME TO documents;

-- Recreate indexes (without strategy_id)
CREATE INDEX idx_documents_id ON documents(id);
CREATE INDEX idx_documents_type ON documents(document_type);
CREATE INDEX idx_documents_updated ON documents(updated_at);
CREATE INDEX idx_documents_phase ON documents(phase);
CREATE INDEX idx_documents_initiative_id ON documents(initiative_id);
CREATE INDEX idx_documents_short_code ON documents(short_code);

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
