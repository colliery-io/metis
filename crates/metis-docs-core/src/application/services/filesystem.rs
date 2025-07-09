use crate::{MetisError, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Filesystem operations service
/// Handles reading/writing documents to disk and computing file hashes
pub struct FilesystemService;

impl FilesystemService {
    /// Read file contents from disk
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        fs::read_to_string(path).map_err(MetisError::Io)
    }

    /// Write file contents to disk
    pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).map_err(MetisError::Io)?;
        }
        
        fs::write(path, content).map_err(MetisError::Io)
    }

    /// Check if file exists
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Compute SHA256 hash of file contents
    pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
        let contents = Self::read_file(path)?;
        let mut hasher = Sha256::new();
        hasher.update(contents.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Compute SHA256 hash of string content
    pub fn compute_content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get file modification time as Unix timestamp
    pub fn get_file_mtime<P: AsRef<Path>>(path: P) -> Result<f64> {
        let metadata = fs::metadata(path).map_err(MetisError::Io)?;
        let mtime = metadata
            .modified()
            .map_err(MetisError::Io)?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| MetisError::ValidationFailed {
                message: "Invalid file modification time".to_string(),
            })?;
        Ok(mtime.as_secs_f64())
    }

    /// Delete a file
    pub fn delete_file<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::remove_file(path).map_err(MetisError::Io)
    }

    /// List all markdown files in a directory recursively
    pub fn find_markdown_files<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
        use walkdir::WalkDir;
        
        let mut files = Vec::new();
        
        for entry in WalkDir::new(dir).follow_links(true) {
            let entry = entry.map_err(|e| MetisError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Walk error: {}", e)
            )))?;
            
            if entry.file_type().is_file() {
                if let Some(path_str) = entry.path().to_str() {
                    if path_str.ends_with(".md") {
                        files.push(path_str.to_string());
                    }
                }
            }
        }
        
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;

    #[test]
    fn test_write_and_read_file() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.md");
        
        let content = "# Test Document\n\nThis is test content.";
        
        // Write file
        FilesystemService::write_file(&file_path, content).expect("Failed to write file");
        
        // Read file
        let read_content = FilesystemService::read_file(&file_path).expect("Failed to read file");
        assert_eq!(content, read_content);
        
        // Check if file exists
        assert!(FilesystemService::file_exists(&file_path));
    }

    #[test]
    fn test_compute_hashes() {
        let content = "# Test Document\n\nThis is test content.";
        
        // Test content hash
        let hash1 = FilesystemService::compute_content_hash(content);
        let hash2 = FilesystemService::compute_content_hash(content);
        assert_eq!(hash1, hash2); // Same content should produce same hash
        
        let different_content = "# Different Document\n\nThis is different content.";
        let hash3 = FilesystemService::compute_content_hash(different_content);
        assert_ne!(hash1, hash3); // Different content should produce different hash
        
        // Test file hash
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.md");
        FilesystemService::write_file(&file_path, content).expect("Failed to write file");
        
        let file_hash = FilesystemService::compute_file_hash(&file_path).expect("Failed to compute file hash");
        assert_eq!(hash1, file_hash); // File hash should match content hash
    }

    #[test]
    fn test_file_operations() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("subdir").join("test.md");
        
        let content = "# Test Document";
        
        // Write file (should create subdirectory)
        FilesystemService::write_file(&file_path, content).expect("Failed to write file");
        assert!(FilesystemService::file_exists(&file_path));
        
        // Get modification time
        let mtime = FilesystemService::get_file_mtime(&file_path).expect("Failed to get mtime");
        assert!(mtime > 0.0);
        
        // Delete file
        FilesystemService::delete_file(&file_path).expect("Failed to delete file");
        assert!(!FilesystemService::file_exists(&file_path));
    }

    #[test]
    fn test_find_markdown_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let base_path = temp_dir.path();
        
        // Create some test files
        let files = vec![
            "doc1.md",
            "subdir/doc2.md", 
            "subdir/nested/doc3.md",
            "not_markdown.txt",
        ];
        
        for file in &files {
            let file_path = base_path.join(file);
            FilesystemService::write_file(&file_path, "# Test").expect("Failed to write file");
        }
        
        // Find markdown files
        let found_files = FilesystemService::find_markdown_files(base_path)
            .expect("Failed to find markdown files");
        
        // Should find 3 .md files, not the .txt file
        assert_eq!(found_files.len(), 3);
        
        // All found files should end with .md
        for file in &found_files {
            assert!(file.ends_with(".md"));
        }
    }
}