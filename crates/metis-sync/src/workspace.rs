//! Minimal config.toml reader for sync operations.
//!
//! Reads just the sync-relevant fields from `.metis/config.toml` without
//! depending on `metis-docs-core`. This keeps the sync domain self-contained.

use crate::orchestration::SyncConfig;
use crate::SyncError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

// ─── Minimal TOML types ─────────────────────────────────────────────────────

/// Minimal representation of config.toml — only the fields we need.
#[derive(Debug, Deserialize)]
struct MinimalConfig {
    workspace: Option<WorkspaceSection>,
    sync: Option<SyncSection>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceSection {
    prefix: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SyncSection {
    upstream_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_synced_commit: Option<String>,
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Read sync-relevant config from `.metis/config.toml`.
///
/// Returns `None` if:
/// - config.toml doesn't exist
/// - No `[sync]` section with `upstream_url`
/// - No `[workspace]` section with `prefix`
///
/// Returns `Some(SyncConfig)` when both workspace prefix and upstream URL
/// are configured (i.e., multi-workspace mode is active).
pub fn read_sync_config(metis_dir: &Path) -> Result<Option<SyncConfig>, SyncError> {
    let config_path = metis_dir.join("config.toml");

    if !config_path.exists() {
        debug!("no config.toml at {}", config_path.display());
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path)?;

    let parsed: MinimalConfig = toml::from_str(&content).map_err(|e| {
        SyncError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to parse config.toml: {}", e),
        ))
    })?;

    let prefix = match parsed.workspace {
        Some(ws) => ws.prefix,
        None => {
            debug!("no [workspace] section in config.toml");
            return Ok(None);
        }
    };

    let sync_section = match parsed.sync {
        Some(s) => s,
        None => {
            debug!("no [sync] section in config.toml");
            return Ok(None);
        }
    };

    Ok(Some(SyncConfig {
        upstream_url: sync_section.upstream_url,
        workspace_prefix: prefix,
        last_synced_commit: sync_section.last_synced_commit,
    }))
}

/// Update `last_synced_commit` in config.toml after a successful push.
///
/// Reads the file, patches the `[sync].last_synced_commit` field, and writes
/// it back. Preserves all other fields/sections.
pub fn update_synced_commit(metis_dir: &Path, commit_sha: &str) -> Result<(), SyncError> {
    let config_path = metis_dir.join("config.toml");

    let content = std::fs::read_to_string(&config_path)?;

    let mut doc: toml::Table = content.parse::<toml::Table>().map_err(|e| {
        SyncError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to parse config.toml for update: {}", e),
        ))
    })?;

    // Ensure [sync] table exists and update last_synced_commit
    let sync_table = doc
        .entry("sync")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));

    if let toml::Value::Table(ref mut table) = sync_table {
        table.insert(
            "last_synced_commit".to_string(),
            toml::Value::String(commit_sha.to_string()),
        );
    }

    let output = toml::to_string_pretty(&doc).map_err(|e| {
        SyncError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to serialize config.toml: {}", e),
        ))
    })?;

    std::fs::write(&config_path, output)?;

    debug!(
        commit = commit_sha,
        "updated last_synced_commit in config.toml"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_config(dir: &Path, content: &str) {
        std::fs::write(dir.join("config.toml"), content).unwrap();
    }

    #[test]
    fn test_read_sync_config_returns_none_when_no_file() {
        let dir = TempDir::new().unwrap();
        let result = read_sync_config(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_read_sync_config_returns_none_when_no_sync_section() {
        let dir = TempDir::new().unwrap();
        write_config(
            dir.path(),
            r#"
[project]
prefix = "METIS"

[workspace]
prefix = "api"
"#,
        );
        let result = read_sync_config(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_read_sync_config_returns_none_when_no_workspace_section() {
        let dir = TempDir::new().unwrap();
        write_config(
            dir.path(),
            r#"
[project]
prefix = "METIS"

[sync]
upstream_url = "git@github.com:org/repo.git"
"#,
        );
        let result = read_sync_config(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_read_sync_config_returns_some_when_configured() {
        let dir = TempDir::new().unwrap();
        write_config(
            dir.path(),
            r#"
[project]
prefix = "METIS"

[workspace]
prefix = "api"

[sync]
upstream_url = "git@github.com:org/repo.git"
"#,
        );

        let result = read_sync_config(dir.path()).unwrap().unwrap();
        assert_eq!(result.upstream_url, "git@github.com:org/repo.git");
        assert_eq!(result.workspace_prefix, "api");
        assert!(result.last_synced_commit.is_none());
    }

    #[test]
    fn test_read_sync_config_with_last_synced_commit() {
        let dir = TempDir::new().unwrap();
        write_config(
            dir.path(),
            r#"
[project]
prefix = "METIS"

[workspace]
prefix = "api"

[sync]
upstream_url = "git@github.com:org/repo.git"
last_synced_commit = "abc123def456abc123def456abc123def456abc1"
"#,
        );

        let result = read_sync_config(dir.path()).unwrap().unwrap();
        assert_eq!(
            result.last_synced_commit,
            Some("abc123def456abc123def456abc123def456abc1".to_string())
        );
    }

    #[test]
    fn test_update_synced_commit_creates_field() {
        let dir = TempDir::new().unwrap();
        write_config(
            dir.path(),
            r#"
[project]
prefix = "METIS"

[workspace]
prefix = "api"

[sync]
upstream_url = "git@github.com:org/repo.git"
"#,
        );

        let sha = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        update_synced_commit(dir.path(), sha).unwrap();

        // Re-read and verify
        let result = read_sync_config(dir.path()).unwrap().unwrap();
        assert_eq!(result.last_synced_commit, Some(sha.to_string()));
    }

    #[test]
    fn test_update_synced_commit_overwrites_existing() {
        let dir = TempDir::new().unwrap();
        write_config(
            dir.path(),
            r#"
[project]
prefix = "METIS"

[workspace]
prefix = "api"

[sync]
upstream_url = "git@github.com:org/repo.git"
last_synced_commit = "0000000000000000000000000000000000000000"
"#,
        );

        let new_sha = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        update_synced_commit(dir.path(), new_sha).unwrap();

        let result = read_sync_config(dir.path()).unwrap().unwrap();
        assert_eq!(result.last_synced_commit, Some(new_sha.to_string()));
    }

    #[test]
    fn test_update_synced_commit_preserves_other_fields() {
        let dir = TempDir::new().unwrap();
        write_config(
            dir.path(),
            r#"
[project]
prefix = "METIS"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true

[workspace]
prefix = "api"
team = "platform"

[sync]
upstream_url = "git@github.com:org/repo.git"
"#,
        );

        let sha = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        update_synced_commit(dir.path(), sha).unwrap();

        // Verify the other fields are preserved
        let content = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
        assert!(content.contains("prefix = \"METIS\""));
        assert!(content.contains("strategies_enabled = false"));
        assert!(content.contains("team = \"platform\""));
        assert!(content.contains("upstream_url = \"git@github.com:org/repo.git\""));
        assert!(content.contains(&format!("last_synced_commit = \"{}\"", sha)));
    }
}
