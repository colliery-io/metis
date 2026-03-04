//! Workspace filesystem migrations.
//!
//! Handles automatic migration of workspace layouts between versions.
//! Currently supports v1→v2 migration (removing the `strategies/` nesting layer).

use std::path::{Path, PathBuf};

/// Result of a workspace migration
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// Whether a migration was performed
    pub migrated: bool,
    /// Items that were moved during migration
    pub moved_items: Vec<MigrationAction>,
    /// Items that were deleted during migration
    pub deleted_items: Vec<PathBuf>,
}

/// A single migration action (move or delete)
#[derive(Debug, Clone)]
pub struct MigrationAction {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Service for migrating workspace filesystem layouts between versions.
pub struct WorkspaceMigrationService;

impl WorkspaceMigrationService {
    /// Run all pending migrations on a workspace.
    ///
    /// Currently only supports v1→v2 (strategy directory flattening).
    /// This is idempotent — running on an already-migrated workspace is a no-op.
    pub fn migrate(metis_dir: &Path) -> std::io::Result<MigrationReport> {
        let strategies_dir = metis_dir.join("strategies");

        if !strategies_dir.exists() || !strategies_dir.is_dir() {
            return Ok(MigrationReport {
                migrated: false,
                moved_items: vec![],
                deleted_items: vec![],
            });
        }

        tracing::info!(
            "Detected v1 workspace layout (strategies/ directory exists). Migrating to v2..."
        );

        let mut moved_items = Vec::new();
        let mut deleted_items = Vec::new();

        let initiatives_dir = metis_dir.join("initiatives");

        // Iterate over each strategy directory (e.g., strategies/NULL/, strategies/PROJ-S-0001/)
        for entry in std::fs::read_dir(&strategies_dir)? {
            let entry = entry?;
            let strategy_path = entry.path();

            if !strategy_path.is_dir() {
                // Delete any stray files (e.g., strategy.md at strategies/ root)
                tracing::info!("Deleting stray file: {}", strategy_path.display());
                std::fs::remove_file(&strategy_path)?;
                deleted_items.push(strategy_path);
                continue;
            }

            // Delete strategy.md if it exists inside the strategy directory
            let strategy_md = strategy_path.join("strategy.md");
            if strategy_md.exists() {
                tracing::info!("Deleting strategy document: {}", strategy_md.display());
                std::fs::remove_file(&strategy_md)?;
                deleted_items.push(strategy_md);
            }

            // Move initiatives from strategies/{id}/initiatives/* up to initiatives/*
            let nested_initiatives_dir = strategy_path.join("initiatives");
            if nested_initiatives_dir.exists() && nested_initiatives_dir.is_dir() {
                for initiative_entry in std::fs::read_dir(&nested_initiatives_dir)? {
                    let initiative_entry = initiative_entry?;
                    let src = initiative_entry.path();
                    let name = initiative_entry.file_name();
                    let dest = initiatives_dir.join(&name);

                    if dest.exists() {
                        tracing::warn!(
                            "Skipping migration of {} — destination {} already exists",
                            src.display(),
                            dest.display()
                        );
                        continue;
                    }

                    // Ensure parent directory exists
                    std::fs::create_dir_all(&initiatives_dir)?;

                    tracing::info!(
                        "Moving initiative: {} -> {}",
                        src.display(),
                        dest.display()
                    );
                    std::fs::rename(&src, &dest)?;

                    moved_items.push(MigrationAction {
                        from: src,
                        to: dest,
                    });
                }
            }

            // Clean up the now-empty strategy directory
            // Use remove_dir (not remove_dir_all) to fail safely if not empty
            if let Err(e) = std::fs::remove_dir(&strategy_path) {
                // If the directory isn't empty, try removing it recursively
                // (it may contain leftover non-initiative files)
                tracing::info!(
                    "Strategy directory {} not empty after migration ({}), removing recursively",
                    strategy_path.display(),
                    e
                );
                std::fs::remove_dir_all(&strategy_path)?;
            }
            deleted_items.push(strategy_path);
        }

        // Remove the now-empty strategies/ directory itself
        if strategies_dir.exists() {
            if let Err(e) = std::fs::remove_dir(&strategies_dir) {
                tracing::warn!(
                    "Could not remove strategies/ directory: {}. Removing recursively.",
                    e
                );
                std::fs::remove_dir_all(&strategies_dir)?;
            }
            deleted_items.push(strategies_dir);
        }

        tracing::info!(
            "Migration complete: moved {} items, deleted {} items",
            moved_items.len(),
            deleted_items.len()
        );

        Ok(MigrationReport {
            migrated: true,
            moved_items,
            deleted_items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_migration_noop_on_v2_workspace() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");
        fs::create_dir_all(&metis_dir).unwrap();

        // No strategies/ directory — should be a no-op
        let report = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(!report.migrated);
        assert!(report.moved_items.is_empty());
        assert!(report.deleted_items.is_empty());
    }

    #[test]
    fn test_migration_idempotent() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");
        fs::create_dir_all(&metis_dir).unwrap();

        // First run: no strategies/ — no-op
        let report1 = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(!report1.migrated);

        // Second run: still no strategies/ — still no-op
        let report2 = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(!report2.migrated);
    }

    #[test]
    fn test_migration_null_strategy_pattern() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");

        // Create v1 layout: strategies/NULL/initiatives/PROJ-I-0001/
        let initiative_dir = metis_dir
            .join("strategies")
            .join("NULL")
            .join("initiatives")
            .join("PROJ-I-0001");
        fs::create_dir_all(&initiative_dir).unwrap();
        fs::write(initiative_dir.join("initiative.md"), "# Test Initiative").unwrap();

        let tasks_dir = initiative_dir.join("tasks");
        fs::create_dir_all(&tasks_dir).unwrap();
        fs::write(tasks_dir.join("PROJ-T-0001.md"), "# Test Task").unwrap();

        let report = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(report.migrated);
        assert_eq!(report.moved_items.len(), 1);

        // Verify new layout
        let new_initiative = metis_dir.join("initiatives").join("PROJ-I-0001");
        assert!(new_initiative.join("initiative.md").exists());
        assert!(new_initiative.join("tasks").join("PROJ-T-0001.md").exists());

        // Verify strategies/ is gone
        assert!(!metis_dir.join("strategies").exists());
    }

    #[test]
    fn test_migration_named_strategy_pattern() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");

        // Create v1 layout with a named strategy
        let strategy_dir = metis_dir.join("strategies").join("PROJ-S-0001");
        fs::create_dir_all(&strategy_dir).unwrap();
        fs::write(strategy_dir.join("strategy.md"), "# Strategy Doc").unwrap();

        let initiative_dir = strategy_dir
            .join("initiatives")
            .join("PROJ-I-0001");
        fs::create_dir_all(&initiative_dir).unwrap();
        fs::write(initiative_dir.join("initiative.md"), "# Initiative").unwrap();

        let report = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(report.migrated);

        // Initiative moved up
        assert!(metis_dir.join("initiatives").join("PROJ-I-0001").join("initiative.md").exists());

        // Strategy doc deleted
        assert!(!strategy_dir.join("strategy.md").exists());

        // strategies/ directory gone
        assert!(!metis_dir.join("strategies").exists());
    }

    #[test]
    fn test_migration_multiple_strategies() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");

        // Create multiple strategies with initiatives
        for (strategy, initiative) in &[
            ("NULL", "PROJ-I-0001"),
            ("PROJ-S-0001", "PROJ-I-0002"),
            ("PROJ-S-0002", "PROJ-I-0003"),
        ] {
            let init_dir = metis_dir
                .join("strategies")
                .join(strategy)
                .join("initiatives")
                .join(initiative);
            fs::create_dir_all(&init_dir).unwrap();
            fs::write(init_dir.join("initiative.md"), format!("# {}", initiative)).unwrap();
        }

        // Add strategy.md files
        fs::write(
            metis_dir.join("strategies").join("PROJ-S-0001").join("strategy.md"),
            "# Strategy 1",
        )
        .unwrap();
        fs::write(
            metis_dir.join("strategies").join("PROJ-S-0002").join("strategy.md"),
            "# Strategy 2",
        )
        .unwrap();

        let report = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(report.migrated);
        assert_eq!(report.moved_items.len(), 3);

        // All initiatives at flat layout
        for init in &["PROJ-I-0001", "PROJ-I-0002", "PROJ-I-0003"] {
            assert!(
                metis_dir.join("initiatives").join(init).join("initiative.md").exists(),
                "Initiative {} should exist at flat layout",
                init
            );
        }

        // strategies/ gone
        assert!(!metis_dir.join("strategies").exists());
    }

    #[test]
    fn test_migration_empty_strategies_dir() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");

        // Create an empty strategies/ directory (no initiatives inside)
        fs::create_dir_all(metis_dir.join("strategies")).unwrap();

        let report = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(report.migrated);
        assert!(report.moved_items.is_empty());

        // strategies/ should still be removed
        assert!(!metis_dir.join("strategies").exists());
    }

    #[test]
    fn test_migration_preserves_existing_initiatives() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");

        // Create an existing v2 initiative
        let existing = metis_dir.join("initiatives").join("PROJ-I-0001");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("initiative.md"), "# Existing").unwrap();

        // Create a v1 initiative with same name (conflict scenario)
        let v1_init = metis_dir
            .join("strategies")
            .join("NULL")
            .join("initiatives")
            .join("PROJ-I-0001");
        fs::create_dir_all(&v1_init).unwrap();
        fs::write(v1_init.join("initiative.md"), "# V1 Version").unwrap();

        // Create a non-conflicting v1 initiative
        let v1_init2 = metis_dir
            .join("strategies")
            .join("NULL")
            .join("initiatives")
            .join("PROJ-I-0002");
        fs::create_dir_all(&v1_init2).unwrap();
        fs::write(v1_init2.join("initiative.md"), "# V1 New").unwrap();

        let report = WorkspaceMigrationService::migrate(&metis_dir).unwrap();
        assert!(report.migrated);

        // Existing initiative should be preserved (not overwritten)
        let content = fs::read_to_string(existing.join("initiative.md")).unwrap();
        assert_eq!(content, "# Existing");

        // Non-conflicting initiative should be moved
        assert!(metis_dir.join("initiatives").join("PROJ-I-0002").join("initiative.md").exists());

        // Only one item moved (the non-conflicting one)
        assert_eq!(report.moved_items.len(), 1);
    }

    #[test]
    fn test_migration_logs_actions() {
        let temp = tempdir().unwrap();
        let metis_dir = temp.path().join(".metis");

        let init_dir = metis_dir
            .join("strategies")
            .join("NULL")
            .join("initiatives")
            .join("PROJ-I-0001");
        fs::create_dir_all(&init_dir).unwrap();
        fs::write(init_dir.join("initiative.md"), "# Test").unwrap();

        let report = WorkspaceMigrationService::migrate(&metis_dir).unwrap();

        // Verify the report contains actionable info
        assert!(report.migrated);
        assert_eq!(report.moved_items.len(), 1);
        assert!(report.moved_items[0].from.to_string_lossy().contains("strategies"));
        assert!(report.moved_items[0].to.to_string_lossy().contains("initiatives"));
        assert!(!report.deleted_items.is_empty());
    }
}
