-- Full-text search index (SQLite, FTS5). Hand-written: the schema generator
-- only emits portable DDL, and FTS is a sanctioned per-backend divergence.
-- Keyed by short_code (TEXT, identical on both backends) so results map back to
-- work_items without depending on the uuid representation. `content` is
-- title + body, fed at write time by the service layer.
CREATE VIRTUAL TABLE work_item_search USING fts5(short_code UNINDEXED, content);
