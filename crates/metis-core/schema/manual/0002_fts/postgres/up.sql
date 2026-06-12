-- Full-text search index (Postgres, tsvector + GIN). Hand-written per-backend
-- counterpart to the SQLite FTS5 table. Keyed by short_code; `content` is
-- title + body fed at write time; `tsv` is derived and indexed for `@@`.
CREATE TABLE work_item_search (
    short_code TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL,
    tsv tsvector GENERATED ALWAYS AS (to_tsvector('english', content)) STORED
);
CREATE INDEX work_item_search_tsv ON work_item_search USING GIN (tsv);
