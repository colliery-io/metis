CREATE TABLE projects (id BLOB PRIMARY KEY NOT NULL, slug TEXT NOT NULL UNIQUE, name TEXT NOT NULL, config TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);

CREATE TABLE users (id BLOB PRIMARY KEY NOT NULL, username TEXT NOT NULL UNIQUE, display_name TEXT NOT NULL, email TEXT, is_admin INTEGER NOT NULL, active INTEGER NOT NULL, created_at TEXT NOT NULL);

CREATE TABLE tokens (id BLOB PRIMARY KEY NOT NULL, user_id BLOB NOT NULL REFERENCES users (id), name TEXT NOT NULL, agent_name TEXT, token_hash TEXT NOT NULL UNIQUE, created_at TEXT NOT NULL, last_used_at TEXT, revoked_at TEXT);

CREATE TABLE repos (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL REFERENCES projects (id), slug TEXT NOT NULL, git_urls TEXT NOT NULL, normalized_urls TEXT NOT NULL, created_at TEXT NOT NULL);

CREATE TABLE objects (object_key TEXT PRIMARY KEY NOT NULL, content BLOB NOT NULL, byte_size BIGINT NOT NULL, created_at TEXT NOT NULL);

CREATE TABLE work_items (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL REFERENCES projects (id), short_code TEXT NOT NULL UNIQUE, seq INTEGER NOT NULL, item_type TEXT NOT NULL, phase TEXT NOT NULL, title TEXT NOT NULL, content_key TEXT, parent_id BLOB, assignee BLOB, created_by BLOB NOT NULL, archived INTEGER NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);

CREATE TABLE work_item_links (from_id BLOB NOT NULL, to_id BLOB NOT NULL, kind TEXT NOT NULL, PRIMARY KEY (from_id, to_id, kind));

CREATE TABLE work_item_repos (item_id BLOB NOT NULL REFERENCES work_items (id), repo_id BLOB NOT NULL REFERENCES repos (id), PRIMARY KEY (item_id, repo_id));

CREATE TABLE work_item_tags (item_id BLOB NOT NULL REFERENCES work_items (id), tag TEXT NOT NULL, PRIMARY KEY (item_id, tag));

CREATE TABLE exit_criteria (id BLOB PRIMARY KEY NOT NULL, item_id BLOB NOT NULL REFERENCES work_items (id), ordinal INTEGER NOT NULL, text TEXT NOT NULL, met INTEGER NOT NULL, met_by BLOB, met_at TEXT);

CREATE TABLE events (id BLOB PRIMARY KEY NOT NULL, item_id BLOB NOT NULL REFERENCES work_items (id), actor_user BLOB NOT NULL, actor_agent TEXT, kind TEXT NOT NULL, payload TEXT NOT NULL, created_at TEXT NOT NULL);

CREATE TABLE views (id BLOB PRIMARY KEY NOT NULL, project_id BLOB NOT NULL REFERENCES projects (id), owner BLOB NOT NULL, name TEXT NOT NULL, query_json TEXT NOT NULL, shared INTEGER NOT NULL);

CREATE INDEX idx_work_items_project ON work_items(project_id);

CREATE INDEX idx_work_items_phase ON work_items(phase);

CREATE INDEX idx_work_items_parent ON work_items(parent_id);

CREATE INDEX idx_work_items_assignee ON work_items(assignee);

CREATE INDEX idx_events_item ON events(item_id);

CREATE INDEX idx_work_item_links_to ON work_item_links(to_id);
