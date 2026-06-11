CREATE TABLE projects (id UUID PRIMARY KEY NOT NULL, slug TEXT NOT NULL UNIQUE, name TEXT NOT NULL, config JSONB NOT NULL, created_at TIMESTAMPTZ NOT NULL, updated_at TIMESTAMPTZ NOT NULL);

CREATE TABLE users (id UUID PRIMARY KEY NOT NULL, username TEXT NOT NULL UNIQUE, display_name TEXT NOT NULL, email TEXT, is_admin BOOLEAN NOT NULL, active BOOLEAN NOT NULL, created_at TIMESTAMPTZ NOT NULL);

CREATE TABLE tokens (id UUID PRIMARY KEY NOT NULL, user_id UUID NOT NULL REFERENCES users (id), name TEXT NOT NULL, agent_name TEXT, token_hash TEXT NOT NULL UNIQUE, created_at TIMESTAMPTZ NOT NULL, last_used_at TIMESTAMPTZ, revoked_at TIMESTAMPTZ);

CREATE TABLE repos (id UUID PRIMARY KEY NOT NULL, project_id UUID NOT NULL REFERENCES projects (id), slug TEXT NOT NULL, git_urls JSONB NOT NULL, normalized_urls JSONB NOT NULL, created_at TIMESTAMPTZ NOT NULL);

CREATE TABLE objects (object_key TEXT PRIMARY KEY NOT NULL, content BYTEA NOT NULL, byte_size BIGINT NOT NULL, created_at TIMESTAMPTZ NOT NULL);

CREATE TABLE work_items (id UUID PRIMARY KEY NOT NULL, project_id UUID NOT NULL REFERENCES projects (id), short_code TEXT NOT NULL UNIQUE, seq INTEGER NOT NULL, item_type TEXT NOT NULL, phase TEXT NOT NULL, title TEXT NOT NULL, content_key TEXT, parent_id UUID, assignee UUID, created_by UUID NOT NULL, archived BOOLEAN NOT NULL, created_at TIMESTAMPTZ NOT NULL, updated_at TIMESTAMPTZ NOT NULL);

CREATE TABLE work_item_links (from_id UUID NOT NULL, to_id UUID NOT NULL, kind TEXT NOT NULL, PRIMARY KEY (from_id, to_id, kind));

CREATE TABLE work_item_repos (item_id UUID NOT NULL REFERENCES work_items (id), repo_id UUID NOT NULL REFERENCES repos (id), PRIMARY KEY (item_id, repo_id));

CREATE TABLE work_item_tags (item_id UUID NOT NULL REFERENCES work_items (id), tag TEXT NOT NULL, PRIMARY KEY (item_id, tag));

CREATE TABLE exit_criteria (id UUID PRIMARY KEY NOT NULL, item_id UUID NOT NULL REFERENCES work_items (id), ordinal INTEGER NOT NULL, text TEXT NOT NULL, met BOOLEAN NOT NULL, met_by UUID, met_at TIMESTAMPTZ);

CREATE TABLE events (id UUID PRIMARY KEY NOT NULL, item_id UUID NOT NULL REFERENCES work_items (id), actor_user UUID NOT NULL, actor_agent TEXT, kind TEXT NOT NULL, payload JSONB NOT NULL, created_at TIMESTAMPTZ NOT NULL);

CREATE TABLE views (id UUID PRIMARY KEY NOT NULL, project_id UUID NOT NULL REFERENCES projects (id), owner UUID NOT NULL, name TEXT NOT NULL, query_json JSONB NOT NULL, shared BOOLEAN NOT NULL);

CREATE INDEX idx_work_items_project ON work_items(project_id);

CREATE INDEX idx_work_items_phase ON work_items(phase);

CREATE INDEX idx_work_items_parent ON work_items(parent_id);

CREATE INDEX idx_work_items_assignee ON work_items(assignee);

CREATE INDEX idx_events_item ON events(item_id);

CREATE INDEX idx_work_item_links_to ON work_item_links(to_id);
