-- Explicit down (the migration creates indexes, so we do not rely on the
-- generator's derived DROP TABLE). Drop in reverse dependency order.
DROP TABLE IF EXISTS views;
DROP TABLE IF EXISTS events;
DROP TABLE IF EXISTS exit_criteria;
DROP TABLE IF EXISTS work_item_tags;
DROP TABLE IF EXISTS work_item_repos;
DROP TABLE IF EXISTS work_item_links;
DROP TABLE IF EXISTS work_items;
DROP TABLE IF EXISTS objects;
DROP TABLE IF EXISTS repos;
DROP TABLE IF EXISTS tokens;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS projects;
