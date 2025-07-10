use std::path::PathBuf;

/// Check if we're in a Metis workspace by walking up the directory tree
///
/// Returns (found, metis_dir_path) where:
/// - found: true if a valid .metis/ vault was found
/// - metis_dir_path: absolute path to the .metis/ directory (if found)
pub fn has_metis_vault() -> (bool, Option<PathBuf>) {
    let Ok(mut current) = std::env::current_dir() else {
        return (false, None);
    };

    loop {
        let metis_dir = current.join(".metis");
        let db_path = metis_dir.join("metis.db");

        // Check if this directory has a valid Metis vault
        if metis_dir.exists() && metis_dir.is_dir() && db_path.exists() && db_path.is_file() {
            return (true, Some(metis_dir));
        }

        // Move up to parent directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            // Reached filesystem root, no workspace found
            break;
        }
    }

    (false, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_has_metis_vault_false_when_no_directory() {
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let (found, _) = has_metis_vault();
        assert!(!found);
    }

    #[test]
    fn test_has_metis_vault_true_when_valid() {
        let temp_dir = tempdir().unwrap();
        let metis_dir = temp_dir.path().join(".metis");
        let db_path = metis_dir.join("metis.db");

        std::fs::create_dir(&metis_dir).unwrap();
        std::fs::write(&db_path, "test").unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let (found, metis_dir_path) = has_metis_vault();
        assert!(found);

        // Canonicalize both paths to handle symlinks (e.g., /var vs /private/var on macOS)
        let returned_path = metis_dir_path.unwrap().canonicalize().unwrap();
        let expected_path = metis_dir.canonicalize().unwrap();
        assert_eq!(returned_path, expected_path);
    }
}
