use crate::application::services::{DatabaseService, FilesystemService};
use crate::dal::database::models::{Document, NewDocument};
use crate::domain::documents::{
    factory::DocumentFactory, traits::Document as DocumentTrait, types::DocumentId,
};
use crate::{MetisError, Result};
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Synchronization service - bridges filesystem and database
pub struct SyncService<'a> {
    db_service: &'a mut DatabaseService,
    workspace_dir: Option<&'a Path>,
    db_path: Option<std::path::PathBuf>,
}

impl<'a> SyncService<'a> {
    pub fn new(db_service: &'a mut DatabaseService) -> Self {
        Self {
            db_service,
            workspace_dir: None,
            db_path: None,
        }
    }

    /// Set the workspace directory for lineage extraction
    /// Note: We store the original path reference without canonicalizing here
    /// because canonicalization requires owned PathBuf. The caller should ensure
    /// paths are properly resolved when needed.
    pub fn with_workspace_dir(mut self, workspace_dir: &'a Path) -> Self {
        self.workspace_dir = Some(workspace_dir);
        // Infer db_path from workspace_dir
        self.db_path = Some(workspace_dir.join("metis.db"));
        self
    }

    /// Convert absolute path to relative path (relative to workspace directory)
    /// Returns the path as-is if workspace_dir is not set or if stripping fails
    fn to_relative_path<P: AsRef<Path>>(&self, absolute_path: P) -> String {
        if let Some(workspace_dir) = self.workspace_dir {
            if let Ok(relative) = absolute_path.as_ref().strip_prefix(workspace_dir) {
                return relative.to_string_lossy().to_string();
            }
        }
        // Fallback to absolute path if no workspace or stripping fails
        absolute_path.as_ref().to_string_lossy().to_string()
    }

    /// Convert relative path to absolute path (prepends workspace directory)
    /// Returns the path as-is if workspace_dir is not set
    fn to_absolute_path(&self, relative_path: &str) -> std::path::PathBuf {
        if let Some(workspace_dir) = self.workspace_dir {
            workspace_dir.join(relative_path)
        } else {
            // Fallback: assume it's already absolute
            std::path::PathBuf::from(relative_path)
        }
    }

    /// Direction 1: File → DocumentObject → Database
    /// Load a document from filesystem and store in database
    pub async fn import_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<Document> {
        // Convert absolute path to relative path for database storage
        let path_str = self.to_relative_path(&file_path);

        // Use DocumentFactory to parse file into domain object
        let document_obj = DocumentFactory::from_file(&file_path).await.map_err(|e| {
            MetisError::ValidationFailed {
                message: format!("Failed to parse document: {}", e),
            }
        })?;

        // Get file metadata
        let file_hash = FilesystemService::compute_file_hash(&file_path)?;
        let updated_at = FilesystemService::get_file_mtime(&file_path)?;
        let content = FilesystemService::read_file(&file_path)?;

        // Convert domain object to database model
        let new_doc = self.domain_to_database_model(
            document_obj.as_ref(),
            &path_str,
            file_hash,
            updated_at,
            content,
        )?;

        // Store in database
        self.db_service.create_document(new_doc)
    }

    /// Direction 2: Database → DocumentObject → File
    /// Export a document from database to filesystem
    pub async fn export_to_file(&mut self, filepath: &str) -> Result<()> {
        // Get document from database (filepath in DB is relative)
        let db_doc = self.db_service.find_by_filepath(filepath)?.ok_or_else(|| {
            MetisError::DocumentNotFound {
                id: filepath.to_string(),
            }
        })?;

        // Get content from database
        let content = db_doc.content.ok_or_else(|| MetisError::ValidationFailed {
            message: "Document has no content".to_string(),
        })?;

        // Convert relative path to absolute for filesystem access
        let absolute_path = self.to_absolute_path(filepath);

        // Write to filesystem
        FilesystemService::write_file(absolute_path, &content)?;

        Ok(())
    }

    /// Convert domain object to database model
    fn domain_to_database_model(
        &self,
        document_obj: &dyn DocumentTrait,
        filepath: &str,
        file_hash: String,
        updated_at: f64,
        content: String,
    ) -> Result<NewDocument> {
        let core = document_obj.core();
        let phase = document_obj
            .phase()
            .map_err(|e| MetisError::ValidationFailed {
                message: format!("Failed to get document phase: {}", e),
            })?
            .to_string();

        // Extract lineage from filesystem path if workspace directory is available
        let (fs_strategy_id, fs_initiative_id) = if let Some(workspace_dir) = self.workspace_dir {
            Self::extract_lineage_from_path(filepath, workspace_dir)
        } else {
            (None, None)
        };

        // Use filesystem lineage if available, otherwise use document lineage
        let final_strategy_id = fs_strategy_id
            .or_else(|| core.strategy_id.clone())
            .map(|id| id.to_string());
        let final_initiative_id = fs_initiative_id
            .or_else(|| core.initiative_id.clone())
            .map(|id| id.to_string());

        Ok(NewDocument {
            filepath: filepath.to_string(),
            id: document_obj.id().to_string(),
            title: core.title.clone(),
            document_type: document_obj.document_type().to_string(),
            created_at: core.metadata.created_at.timestamp() as f64,
            updated_at,
            archived: core.archived,
            exit_criteria_met: document_obj.exit_criteria_met(),
            file_hash,
            frontmatter_json: serde_json::to_string(&core.metadata).map_err(MetisError::Json)?,
            content: Some(content),
            phase,
            strategy_id: final_strategy_id,
            initiative_id: final_initiative_id,
            short_code: core.metadata.short_code.clone(),
        })
    }

    /// Extract lineage information from file path
    /// Returns (strategy_id, initiative_id) based on filesystem structure
    fn extract_lineage_from_path<P: AsRef<Path>>(
        file_path: P,
        workspace_dir: &Path,
    ) -> (Option<DocumentId>, Option<DocumentId>) {
        let path = file_path.as_ref();

        // Get relative path from workspace
        let relative_path = match path.strip_prefix(workspace_dir) {
            Ok(rel) => rel,
            Err(_) => return (None, None),
        };

        let path_parts: Vec<&str> = relative_path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        // Match the path structure
        match path_parts.as_slice() {
            // strategies/{strategy-id}/strategy.md
            ["strategies", strategy_id, "strategy.md"] => {
                if strategy_id == &"NULL" {
                    (None, None)
                } else {
                    (Some(DocumentId::from(*strategy_id)), None)
                }
            }
            // strategies/{strategy-id}/initiatives/{initiative-id}/initiative.md
            ["strategies", strategy_id, "initiatives", initiative_id, "initiative.md"] => {
                let strat_id = if strategy_id == &"NULL" {
                    None
                } else {
                    Some(DocumentId::from(*strategy_id))
                };
                let init_id = if initiative_id == &"NULL" {
                    None
                } else {
                    Some(DocumentId::from(*initiative_id))
                };
                (strat_id, init_id)
            }
            // strategies/{strategy-id}/initiatives/{initiative-id}/tasks/{task-id}.md
            ["strategies", strategy_id, "initiatives", initiative_id, "tasks", _] => {
                let strat_id = if strategy_id == &"NULL" {
                    None
                } else {
                    Some(DocumentId::from(*strategy_id))
                };
                let init_id = if initiative_id == &"NULL" {
                    None
                } else {
                    Some(DocumentId::from(*initiative_id))
                };
                (strat_id, init_id)
            }
            // backlog/{task-id}.md (no lineage)
            ["backlog", _] => (None, None),
            // adrs/{adr-id}.md (no lineage)
            ["adrs", _] => (None, None),
            // vision.md (no lineage)
            ["vision.md"] => (None, None),
            // Default: no lineage
            _ => (None, None),
        }
    }

    /// Extract document short code from file without keeping the document object around
    fn extract_document_short_code<P: AsRef<Path>>(file_path: P) -> Result<String> {
        // Read file content to extract frontmatter and get document short code
        let raw_content = std::fs::read_to_string(file_path.as_ref()).map_err(|e| {
            MetisError::ValidationFailed {
                message: format!("Failed to read file: {}", e),
            }
        })?;

        // Parse frontmatter to get document short code
        use gray_matter::{engine::YAML, Matter};
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&raw_content);

        // Extract short_code from frontmatter
        if let Some(frontmatter) = result.data {
            let fm_map = match frontmatter {
                gray_matter::Pod::Hash(map) => map,
                _ => {
                    return Err(MetisError::ValidationFailed {
                        message: "Frontmatter must be a hash/map".to_string(),
                    });
                }
            };

            if let Some(gray_matter::Pod::String(short_code_str)) = fm_map.get("short_code") {
                return Ok(short_code_str.clone());
            }
        }

        Err(MetisError::ValidationFailed {
            message: "Document missing short_code in frontmatter".to_string(),
        })
    }

    /// Update a document that has been moved to a new path
    async fn update_moved_document<P: AsRef<Path>>(
        &mut self,
        existing_doc: &Document,
        new_file_path: P,
    ) -> Result<()> {
        // Delete the old database entry first (to handle foreign key constraints)
        self.db_service.delete_document(&existing_doc.filepath)?;

        // Import the document at the new path
        self.import_from_file(&new_file_path).await?;

        Ok(())
    }

    /// Detect and resolve short code collisions across all markdown files
    /// Returns list of renumbering results
    async fn resolve_short_code_collisions<P: AsRef<Path>>(
        &mut self,
        dir_path: P,
    ) -> Result<Vec<SyncResult>> {
        let mut results = Vec::new();

        // Step 0: Update counters from filesystem FIRST
        // This ensures the counter knows about all existing short codes before we generate new ones
        self.update_counters_from_filesystem(&dir_path)?;

        // Step 1: Scan all markdown files and group by short code
        let files = FilesystemService::find_markdown_files(&dir_path)?;
        let mut short_code_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for file_path in files {
            match Self::extract_document_short_code(&file_path) {
                Ok(short_code) => {
                    short_code_map
                        .entry(short_code)
                        .or_default()
                        .push(PathBuf::from(&file_path));
                }
                Err(e) => {
                    tracing::warn!("Failed to extract short code from {}: {}", file_path, e);
                }
            }
        }

        // Step 2: Find collisions (short codes with multiple files)
        let mut collision_groups: Vec<(String, Vec<PathBuf>)> = short_code_map
            .into_iter()
            .filter(|(_, paths)| paths.len() > 1)
            .collect();

        if collision_groups.is_empty() {
            return Ok(results);
        }

        // Step 3: Sort collision groups by path depth (resolve parents first)
        for (_, paths) in &mut collision_groups {
            paths.sort_by(|a, b| {
                let depth_a = a.components().count();
                let depth_b = b.components().count();
                depth_a.cmp(&depth_b).then_with(|| a.cmp(b))
            });
        }

        // Step 4: Resolve each collision group
        for (old_short_code, mut paths) in collision_groups {
            tracing::info!(
                "Detected short code collision for {}: {} files",
                old_short_code,
                paths.len()
            );

            // First path keeps original short code, rest get renumbered
            let _keeper = paths.remove(0);

            for path in paths {
                match self.renumber_document(&path, &old_short_code).await {
                    Ok(new_short_code) => {
                        let relative_path = self.to_relative_path(&path);
                        results.push(SyncResult::Renumbered {
                            filepath: relative_path,
                            old_short_code: old_short_code.clone(),
                            new_short_code,
                        });
                    }
                    Err(e) => {
                        let relative_path = self.to_relative_path(&path);
                        results.push(SyncResult::Error {
                            filepath: relative_path,
                            error: format!("Failed to renumber: {}", e),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Renumber a single document to resolve short code collision
    /// Returns the new short code
    async fn renumber_document<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        old_short_code: &str,
    ) -> Result<String> {
        let file_path = file_path.as_ref();

        // Step 1: Read current document content
        let content = FilesystemService::read_file(file_path)?;

        // Step 2: Parse frontmatter
        use gray_matter::{engine::YAML, Matter};
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(&content);

        // Step 3: Extract document type from frontmatter to generate new short code
        let doc_type = if let Some(frontmatter) = &parsed.data {
            if let gray_matter::Pod::Hash(map) = frontmatter {
                if let Some(gray_matter::Pod::String(level_str)) = map.get("level") {
                    level_str.as_str()
                } else {
                    return Err(MetisError::ValidationFailed {
                        message: "Document missing 'level' in frontmatter".to_string(),
                    });
                }
            } else {
                return Err(MetisError::ValidationFailed {
                    message: "Frontmatter must be a hash/map".to_string(),
                });
            }
        } else {
            return Err(MetisError::ValidationFailed {
                message: "Document missing frontmatter".to_string(),
            });
        };

        // Step 4: Generate new short code
        let db_path_str = self
            .db_path
            .as_ref()
            .ok_or_else(|| MetisError::ValidationFailed {
                message: "Database path not set".to_string(),
            })?
            .to_string_lossy()
            .to_string();

        use crate::dal::database::configuration_repository::ConfigurationRepository;
        use diesel::sqlite::SqliteConnection;
        use diesel::Connection;

        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&db_path_str).map_err(|e| {
                MetisError::ConfigurationError(
                    crate::domain::configuration::ConfigurationError::InvalidValue(e.to_string()),
                )
            })?,
        );

        let new_short_code = config_repo.generate_short_code(doc_type)?;

        // Step 5: Update frontmatter with new short code using regex
        let short_code_pattern = regex::Regex::new(r#"(?m)^short_code:\s*['"]?([^'"]+)['"]?$"#)
            .map_err(|e| MetisError::ValidationFailed {
                message: format!("Failed to compile regex: {}", e),
            })?;

        let updated_content = short_code_pattern.replace(
            &content,
            format!("short_code: \"{}\"", new_short_code)
        ).to_string();

        // Step 6: Update cross-references in sibling documents
        self.update_sibling_references(file_path, old_short_code, &new_short_code)
            .await?;

        // Step 7: Write updated content back to file
        FilesystemService::write_file(file_path, &updated_content)?;

        // Step 8: Rename file if filename contains the short code
        // Extract just the suffix (e.g., "T-0001" from "TEST-T-0001")
        let old_suffix = old_short_code.rsplit('-').take(2).collect::<Vec<_>>();
        let old_suffix = format!("{}-{}", old_suffix[1], old_suffix[0]);
        let new_suffix = new_short_code.rsplit('-').take(2).collect::<Vec<_>>();
        let new_suffix = format!("{}-{}", new_suffix[1], new_suffix[0]);

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| MetisError::ValidationFailed {
                message: "Invalid file path".to_string(),
            })?;

        if file_name.contains(&old_suffix) {
            let new_file_name = file_name.replace(&old_suffix, &new_suffix);
            let new_path = file_path.with_file_name(new_file_name);
            std::fs::rename(file_path, &new_path)?;

            tracing::info!(
                "Renumbered {} from {} to {}",
                file_path.display(),
                old_short_code,
                new_short_code
            );
        }

        Ok(new_short_code)
    }

    /// Update cross-references in sibling documents (same directory)
    async fn update_sibling_references<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        old_short_code: &str,
        new_short_code: &str,
    ) -> Result<()> {
        let file_path = file_path.as_ref();

        // Get parent directory (sibling group)
        let parent_dir = file_path.parent().ok_or_else(|| MetisError::ValidationFailed {
            message: "File has no parent directory".to_string(),
        })?;

        // Find all markdown files in same directory
        let siblings = FilesystemService::find_markdown_files(parent_dir)?;

        // Create regex pattern to match short code as whole word
        let pattern_str = format!(r"\b{}\b", regex::escape(old_short_code));
        let pattern = regex::Regex::new(&pattern_str)
            .map_err(|e| MetisError::ValidationFailed {
                message: format!("Failed to compile regex: {}", e),
            })?;

        // Update each sibling file
        for sibling_path in siblings {
            let sibling_path_buf = PathBuf::from(&sibling_path);
            if sibling_path_buf == file_path {
                continue; // Skip the document we just renumbered
            }

            match FilesystemService::read_file(&sibling_path) {
                Ok(content) => {
                    if pattern.is_match(&content) {
                        let updated_content = pattern.replace_all(&content, new_short_code);
                        if let Err(e) = FilesystemService::write_file(&sibling_path, &updated_content) {
                            tracing::warn!(
                                "Failed to update references in {}: {}",
                                sibling_path,
                                e
                            );
                        } else {
                            tracing::info!(
                                "Updated references in {} from {} to {}",
                                sibling_path,
                                old_short_code,
                                new_short_code
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read sibling file {}: {}", sibling_path, e);
                }
            }
        }

        Ok(())
    }

    /// Synchronize a single file between filesystem and database using directional methods
    pub async fn sync_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<SyncResult> {
        // Convert absolute path to relative for database queries
        let relative_path_str = self.to_relative_path(&file_path);

        // Check if file exists on filesystem
        let file_exists = FilesystemService::file_exists(&file_path);

        // Check if document exists in database at this filepath (DB stores relative paths)
        let db_doc_by_path = self.db_service.find_by_filepath(&relative_path_str)?;

        match (file_exists, db_doc_by_path) {
            // File exists, not in database at this path - need to check if it's a moved document
            (true, None) => {
                // Extract the document short code without creating full document object
                let short_code = Self::extract_document_short_code(&file_path)?;

                // Check if a document with this short code exists at a different path
                if let Some(existing_doc) = self.db_service.find_by_short_code(&short_code)? {
                    // Document moved - update the existing record
                    let old_path = existing_doc.filepath.clone();
                    self.update_moved_document(&existing_doc, &file_path)
                        .await?;
                    Ok(SyncResult::Moved {
                        from: old_path,
                        to: relative_path_str,
                    })
                } else {
                    // Truly new document - import it
                    self.import_from_file(&file_path).await?;
                    Ok(SyncResult::Imported {
                        filepath: relative_path_str,
                    })
                }
            }

            // File doesn't exist, but in database - remove from database
            (false, Some(_)) => {
                self.db_service.delete_document(&relative_path_str)?;
                Ok(SyncResult::Deleted {
                    filepath: relative_path_str,
                })
            }

            // Both exist - check if file changed
            (true, Some(db_doc)) => {
                let current_hash = FilesystemService::compute_file_hash(&file_path)?;

                if db_doc.file_hash != current_hash {
                    // File changed, reimport (file is source of truth)
                    self.db_service.delete_document(&relative_path_str)?;
                    self.import_from_file(&file_path).await?;
                    Ok(SyncResult::Updated {
                        filepath: relative_path_str,
                    })
                } else {
                    Ok(SyncResult::UpToDate {
                        filepath: relative_path_str,
                    })
                }
            }

            // Neither exists
            (false, None) => Ok(SyncResult::NotFound {
                filepath: relative_path_str,
            }),
        }
    }

    /// Sync all markdown files in a directory
    pub async fn sync_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<Vec<SyncResult>> {
        let mut results = Vec::new();

        // Step 1: Detect and resolve short code collisions BEFORE syncing to database
        // This ensures we don't try to import duplicate short codes
        let collision_results = self.resolve_short_code_collisions(&dir_path).await?;
        results.extend(collision_results);

        // Step 2: Re-scan all markdown files AFTER renumbering
        // This picks up renamed files with new short codes
        let files = FilesystemService::find_markdown_files(&dir_path)?;

        // Step 3: Sync each file
        for file_path in files {
            match self.sync_file(&file_path).await {
                Ok(result) => results.push(result),
                Err(e) => results.push(SyncResult::Error {
                    filepath: file_path,
                    error: e.to_string(),
                }),
            }
        }

        // Step 4: Check for orphaned database entries (files that were deleted)
        let db_pairs = self.db_service.get_all_id_filepath_pairs()?;
        for (_, relative_filepath) in db_pairs {
            // Convert relative path from DB to absolute for filesystem check
            let absolute_path = self.to_absolute_path(&relative_filepath);
            if !FilesystemService::file_exists(&absolute_path) {
                // File no longer exists, delete from database
                match self.db_service.delete_document(&relative_filepath) {
                    Ok(_) => results.push(SyncResult::Deleted {
                        filepath: relative_filepath,
                    }),
                    Err(e) => results.push(SyncResult::Error {
                        filepath: relative_filepath,
                        error: e.to_string(),
                    }),
                }
            }
        }

        // Step 5: Update counters based on max seen values
        self.update_counters_from_filesystem(&dir_path)?;

        Ok(results)
    }

    /// Verify database and filesystem are in sync
    pub fn verify_sync<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<Vec<SyncIssue>> {
        let mut issues = Vec::new();

        // Find all markdown files (returns absolute paths)
        let files = FilesystemService::find_markdown_files(&dir_path)?;

        // Check each file
        for file_path in &files {
            // Convert absolute path to relative for DB query
            let relative_path = self.to_relative_path(file_path);

            if let Some(db_doc) = self.db_service.find_by_filepath(&relative_path)? {
                let current_hash = FilesystemService::compute_file_hash(file_path)?;
                if db_doc.file_hash != current_hash {
                    issues.push(SyncIssue::OutOfSync {
                        filepath: relative_path,
                        reason: "File hash mismatch".to_string(),
                    });
                }
            } else {
                issues.push(SyncIssue::MissingFromDatabase {
                    filepath: relative_path,
                });
            }
        }

        // Check for orphaned database entries
        let db_pairs = self.db_service.get_all_id_filepath_pairs()?;
        for (_, relative_filepath) in db_pairs {
            // Convert relative path from DB to absolute for filesystem check
            let absolute_path = self.to_absolute_path(&relative_filepath);
            let absolute_str = absolute_path.to_string_lossy().to_string();
            if !files.contains(&absolute_str) && !FilesystemService::file_exists(&absolute_path) {
                issues.push(SyncIssue::MissingFromFilesystem {
                    filepath: relative_filepath,
                });
            }
        }

        Ok(issues)
    }

    /// Update counters in database based on max values seen in filesystem
    /// Called after collision resolution to ensure counters are up to date
    fn update_counters_from_filesystem<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<()> {
        let counters = self.recover_counters_from_filesystem(dir_path)?;

        let db_path_str = self
            .db_path
            .as_ref()
            .ok_or_else(|| MetisError::ValidationFailed {
                message: "Database path not set".to_string(),
            })?
            .to_string_lossy()
            .to_string();

        use crate::dal::database::configuration_repository::ConfigurationRepository;
        use diesel::sqlite::SqliteConnection;
        use diesel::Connection;

        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(&db_path_str).map_err(|e| {
                MetisError::ConfigurationError(
                    crate::domain::configuration::ConfigurationError::InvalidValue(e.to_string()),
                )
            })?,
        );

        for (doc_type, max_counter) in counters {
            // Set counter to max seen value (get_next_short_code_number adds 1)
            config_repo.set_counter_if_lower(&doc_type, max_counter)?;
        }

        Ok(())
    }

    /// Recover short code counters from filesystem by scanning all documents
    ///
    /// This should only be called when:
    /// - Database is missing or corrupt
    /// - Explicit recovery is requested by user
    ///
    /// Returns a map of document type to the highest counter found
    pub fn recover_counters_from_filesystem<P: AsRef<Path>>(
        &self,
        dir_path: P,
    ) -> Result<std::collections::HashMap<String, u32>> {
        use gray_matter::{engine::YAML, Matter};
        use std::collections::HashMap;

        let mut counters: HashMap<String, u32> = HashMap::new();
        let mut skipped_files = 0;
        let mut invalid_short_codes = 0;

        let dir_path = dir_path.as_ref();

        // Guard: Ensure directory exists
        if !dir_path.exists() {
            tracing::warn!("Counter recovery: directory does not exist: {}", dir_path.display());
            return Ok(counters);
        }

        // Find all markdown files
        let files = FilesystemService::find_markdown_files(&dir_path)?;
        tracing::info!("Counter recovery: scanning {} markdown files", files.len());

        for file_path in files {
            // Guard: Read file with error handling
            let content = match std::fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Counter recovery: skipping unreadable file {}: {}", file_path, e);
                    skipped_files += 1;
                    continue;
                }
            };

            // Parse frontmatter
            let matter = Matter::<YAML>::new();
            let result = matter.parse(&content);

            if let Some(frontmatter) = result.data {
                let fm_map = match frontmatter {
                    gray_matter::Pod::Hash(map) => map,
                    _ => continue,
                };

                // Extract short_code
                if let Some(gray_matter::Pod::String(short_code)) = fm_map.get("short_code") {
                    // Guard: Validate format
                    if !Self::is_valid_short_code_format(short_code) {
                        tracing::warn!(
                            "Counter recovery: invalid short code '{}' in {}",
                            short_code,
                            file_path
                        );
                        invalid_short_codes += 1;
                        continue;
                    }

                    // Parse: PREFIX-TYPE-NNNN
                    if let Some((_, type_and_num)) = short_code.split_once('-') {
                        if let Some((type_letter, num_str)) = type_and_num.split_once('-') {
                            let doc_type = match type_letter {
                                "V" => "vision",
                                "S" => "strategy",
                                "I" => "initiative",
                                "T" => "task",
                                "A" => "adr",
                                _ => continue,
                            };

                            // Guard: Parse and validate number
                            match num_str.parse::<u32>() {
                                Ok(num) if num <= 1_000_000 => {
                                    counters
                                        .entry(doc_type.to_string())
                                        .and_modify(|max| {
                                            if num > *max {
                                                *max = num;
                                            }
                                        })
                                        .or_insert(num);
                                }
                                Ok(num) => {
                                    tracing::warn!(
                                        "Counter recovery: suspiciously large counter {} in {}, skipping",
                                        num,
                                        file_path
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Counter recovery: invalid number '{}' in {}: {}",
                                        num_str,
                                        file_path,
                                        e
                                    );
                                    invalid_short_codes += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if skipped_files > 0 || invalid_short_codes > 0 {
            tracing::warn!(
                "Counter recovery: {} files skipped, {} invalid short codes",
                skipped_files,
                invalid_short_codes
            );
        }

        tracing::info!("Recovered counters: {:?}", counters);
        Ok(counters)
    }

    /// Validate short code format: PREFIX-TYPE-NNNN
    fn is_valid_short_code_format(short_code: &str) -> bool {
        let parts: Vec<&str> = short_code.split('-').collect();
        if parts.len() != 3 {
            return false;
        }

        let prefix = parts[0];
        let type_letter = parts[1];
        let number = parts[2];

        // Prefix: 2-8 uppercase letters
        if prefix.len() < 2 || prefix.len() > 8 || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
            return false;
        }

        // Type: single letter from allowed set
        if !matches!(type_letter, "V" | "S" | "I" | "T" | "A") {
            return false;
        }

        // Number: exactly 4 digits
        number.len() == 4 && number.chars().all(|c| c.is_ascii_digit())
    }
}

/// Result of synchronizing a single document
#[derive(Debug, Clone, PartialEq)]
pub enum SyncResult {
    Imported { filepath: String },
    Updated { filepath: String },
    Deleted { filepath: String },
    UpToDate { filepath: String },
    NotFound { filepath: String },
    Error { filepath: String, error: String },
    Moved { from: String, to: String },
    Renumbered {
        filepath: String,
        old_short_code: String,
        new_short_code: String
    },
}

impl SyncResult {
    /// Get the filepath for this result
    pub fn filepath(&self) -> &str {
        match self {
            SyncResult::Imported { filepath }
            | SyncResult::Updated { filepath }
            | SyncResult::Deleted { filepath }
            | SyncResult::UpToDate { filepath }
            | SyncResult::NotFound { filepath }
            | SyncResult::Renumbered { filepath, .. }
            | SyncResult::Error { filepath, .. } => filepath,
            SyncResult::Moved { to, .. } => to,
        }
    }

    /// Check if this result represents a change
    pub fn is_change(&self) -> bool {
        matches!(
            self,
            SyncResult::Imported { .. }
                | SyncResult::Updated { .. }
                | SyncResult::Deleted { .. }
                | SyncResult::Moved { .. }
                | SyncResult::Renumbered { .. }
        )
    }

    /// Check if this result represents an error
    pub fn is_error(&self) -> bool {
        matches!(self, SyncResult::Error { .. })
    }
}

/// Issues found during sync verification
#[derive(Debug, Clone)]
pub enum SyncIssue {
    MissingFromDatabase { filepath: String },
    MissingFromFilesystem { filepath: String },
    OutOfSync { filepath: String, reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::Database;
    use tempfile::tempdir;

    fn setup_services() -> (tempfile::TempDir, DatabaseService) {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        // Use metis.db to match what sync_service expects in with_workspace_dir
        let db_path = temp_dir.path().join("metis.db");
        let db = Database::new(db_path.to_str().unwrap()).expect("Failed to create test database");
        // Initialize configuration with test prefix
        let mut config_repo = db.configuration_repository().expect("Failed to create config repo");
        config_repo.set_project_prefix("TEST").expect("Failed to set prefix");
        let db_service = DatabaseService::new(db.into_repository());
        (temp_dir, db_service)
    }

    fn create_test_document_content() -> String {
        "---\n".to_string()
            + "id: test-document\n"
            + "title: Test Document\n"
            + "level: vision\n"
            + "created_at: \"2021-01-01T00:00:00Z\"\n"
            + "updated_at: \"2021-01-01T00:00:00Z\"\n"
            + "archived: false\n"
            + "short_code: TEST-V-9003\n"
            + "exit_criteria_met: false\n"
            + "tags:\n"
            + "  - \"#phase/draft\"\n"
            + "---\n\n"
            + "# Test Document\n\n"
            + "Test content.\n"
    }

    #[tokio::test]
    async fn test_import_from_file() {
        let (temp_dir, mut db_service) = setup_services();
        let mut sync_service = SyncService::new(&mut db_service);

        let file_path = temp_dir.path().join("test.md");
        FilesystemService::write_file(&file_path, &create_test_document_content())
            .expect("Failed to write file");

        let doc = sync_service
            .import_from_file(&file_path)
            .await
            .expect("Failed to import");
        assert_eq!(doc.title, "Test Document");
        assert_eq!(doc.document_type, "vision");

        // Verify it's in the database
        assert!(db_service
            .document_exists(&file_path.to_string_lossy())
            .expect("Failed to check"));
    }

    #[tokio::test]
    async fn test_sync_file_operations() {
        let (temp_dir, mut db_service) = setup_services();
        let mut sync_service = SyncService::new(&mut db_service);

        let file_path = temp_dir.path().join("test.md");
        let path_str = file_path.to_string_lossy().to_string();

        // Test sync when file doesn't exist
        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::NotFound {
                filepath: path_str.clone()
            }
        );

        // Create file and sync
        FilesystemService::write_file(&file_path, &create_test_document_content())
            .expect("Failed to write file");

        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::Imported {
                filepath: path_str.clone()
            }
        );

        // Sync again - should be up to date
        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::UpToDate {
                filepath: path_str.clone()
            }
        );

        // Modify file
        let modified_content =
            &create_test_document_content().replace("Test content.", "Modified content.");
        FilesystemService::write_file(&file_path, modified_content).expect("Failed to write");

        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::Updated {
                filepath: path_str.clone()
            }
        );

        // Delete file
        FilesystemService::delete_file(&file_path).expect("Failed to delete");

        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::Deleted {
                filepath: path_str.clone()
            }
        );

        // Verify it's gone from database
        assert!(!db_service
            .document_exists(&path_str)
            .expect("Failed to check"));
    }

    #[tokio::test]
    async fn test_sync_directory() {
        let (temp_dir, mut db_service) = setup_services();
        let mut sync_service = SyncService::new(&mut db_service).with_workspace_dir(temp_dir.path());

        // Create multiple files
        let files = vec![
            ("doc1.md", "test-1"),
            ("subdir/doc2.md", "test-2"),
            ("subdir/nested/doc3.md", "test-3"),
        ];

        for (i, (file_path, id)) in files.iter().enumerate() {
            let full_path = temp_dir.path().join(file_path);
            let content = &create_test_document_content()
                .replace("Test Document", &format!("Test Document {}", id))
                .replace("test-document", id)
                .replace("TEST-V-9003", &format!("TEST-V-900{}", i + 3));
            FilesystemService::write_file(&full_path, content).expect("Failed to write");
        }

        // Sync directory
        let results = sync_service
            .sync_directory(temp_dir.path())
            .await
            .expect("Failed to sync directory");

        // Should have 3 imports
        let imports = results
            .iter()
            .filter(|r| matches!(r, SyncResult::Imported { .. }))
            .count();
        assert_eq!(imports, 3);

        // Sync again - all should be up to date
        let results = sync_service
            .sync_directory(temp_dir.path())
            .await
            .expect("Failed to sync directory");
        let up_to_date = results
            .iter()
            .filter(|r| matches!(r, SyncResult::UpToDate { .. }))
            .count();
        assert_eq!(up_to_date, 3);

        // Check that we have results for all files
        // Note: with workspace_dir set, sync returns relative paths
        for (file_path, _) in &files {
            assert!(
                results.iter().any(|r| r.filepath() == *file_path),
                "Expected to find result for {}, but results were: {:?}",
                file_path,
                results.iter().map(|r| r.filepath()).collect::<Vec<_>>()
            );
        }
    }
}
