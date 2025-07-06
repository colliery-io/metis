-- Initial schema for Metis document storage system
-- Based on Storage & Indexing System initiative design

-- Core document metadata table
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY,
    filepath TEXT NOT NULL UNIQUE,
    document_type TEXT NOT NULL,
    level TEXT NOT NULL,
    status TEXT NOT NULL,
    parent_id TEXT,
    created_at REAL NOT NULL,
    updated_at REAL NOT NULL,
    content_hash TEXT NOT NULL,
    frontmatter_json TEXT NOT NULL,
    exit_criteria_met BOOLEAN DEFAULT FALSE,
    content TEXT,
    file_size INTEGER,
    file_modified_at REAL,
    FOREIGN KEY (parent_id) REFERENCES documents(id)
);

-- Document relationships (many-to-many)
CREATE TABLE IF NOT EXISTS document_relationships (
    from_id TEXT NOT NULL,
    to_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL,
    created_at REAL NOT NULL,
    PRIMARY KEY (from_id, to_id, relationship_type),
    FOREIGN KEY (from_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (to_id) REFERENCES documents(id) ON DELETE CASCADE
);

-- Document properties for efficient filtering
CREATE TABLE IF NOT EXISTS document_properties (
    document_id TEXT NOT NULL,
    property_name TEXT NOT NULL,
    property_value TEXT,
    property_type TEXT,
    PRIMARY KEY (document_id, property_name),
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);


-- Full-text search using FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS document_search USING fts5(
    document_id UNINDEXED,
    content,
    title,
    document_type,
    status,
    tokenize='porter unicode61'
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(document_type);
CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status);
CREATE INDEX IF NOT EXISTS idx_documents_parent ON documents(parent_id);
CREATE INDEX IF NOT EXISTS idx_documents_updated ON documents(updated_at);
CREATE INDEX IF NOT EXISTS idx_properties_name ON document_properties(property_name);
CREATE INDEX IF NOT EXISTS idx_properties_value ON document_properties(property_value);
CREATE INDEX IF NOT EXISTS idx_relationships_from ON document_relationships(from_id);
CREATE INDEX IF NOT EXISTS idx_relationships_to ON document_relationships(to_id);

-- Triggers to keep FTS index in sync with documents table
CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
    INSERT INTO document_search(document_id, content, title, document_type, status)
    VALUES (
        new.id,
        new.content,
        json_extract(new.frontmatter_json, '$.title'),
        new.document_type,
        new.status
    );
END;

CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
    UPDATE document_search 
    SET content = new.content,
        title = json_extract(new.frontmatter_json, '$.title'),
        document_type = new.document_type,
        status = new.status
    WHERE document_id = new.id;
END;

CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
    DELETE FROM document_search WHERE document_id = old.id;
END;