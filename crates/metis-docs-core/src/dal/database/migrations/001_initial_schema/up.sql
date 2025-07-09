-- Initial schema for Metis document storage system
-- Diesel-based DAL implementation

-- Core document metadata table
CREATE TABLE documents (
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
    UNIQUE(document_type, id)
);

-- Document parent-child relationships for decomposition hierarchy
CREATE TABLE document_relationships (
    child_id TEXT NOT NULL,
    parent_id TEXT NOT NULL,
    child_filepath TEXT NOT NULL,
    parent_filepath TEXT NOT NULL,
    PRIMARY KEY (child_filepath, parent_filepath),
    FOREIGN KEY (child_filepath) REFERENCES documents(filepath) ON DELETE CASCADE,
    FOREIGN KEY (parent_filepath) REFERENCES documents(filepath) ON DELETE CASCADE
);

-- Document tags for efficient tag-based searching
CREATE TABLE document_tags (
    document_filepath TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (document_filepath, tag),
    FOREIGN KEY (document_filepath) REFERENCES documents(filepath) ON DELETE CASCADE
);

-- Full-text search using FTS5
CREATE VIRTUAL TABLE document_search USING fts5(
    document_filepath UNINDEXED,
    content,
    title,
    document_type,
    tokenize='porter unicode61'
);

-- Indexes for common queries
CREATE INDEX idx_documents_id ON documents(id);
CREATE INDEX idx_documents_type ON documents(document_type);
CREATE INDEX idx_documents_updated ON documents(updated_at);
CREATE INDEX idx_relationships_child_id ON document_relationships(child_id);
CREATE INDEX idx_relationships_parent_id ON document_relationships(parent_id);
CREATE INDEX idx_relationships_child_filepath ON document_relationships(child_filepath);
CREATE INDEX idx_relationships_parent_filepath ON document_relationships(parent_filepath);
CREATE INDEX idx_tags_tag ON document_tags(tag);

-- Triggers to keep FTS index in sync with documents table
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