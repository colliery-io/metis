use crate::application::services::DatabaseService;
use crate::domain::documents::traits::Document;
use crate::domain::documents::types::DocumentType;
use crate::Result;
use crate::{Adr, Initiative, MetisError, Strategy, Task, Vision};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Service for discovering documents by ID across all document types
pub struct DocumentDiscoveryService {
    workspace_dir: PathBuf,
}

/// Result of document discovery
#[derive(Debug)]
pub struct DocumentDiscoveryResult {
    pub document_type: DocumentType,
    pub file_path: PathBuf,
}

impl DocumentDiscoveryService {
    /// Create a new document discovery service for a workspace
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Self {
        let path = workspace_dir.as_ref();

        // Ensure we have an absolute path first
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        };

        // Then canonicalize to handle symlinks (e.g., /tmp vs /private/tmp)
        let workspace_dir = absolute_path
            .canonicalize()
            .unwrap_or(absolute_path);

        Self {
            workspace_dir,
        }
    }

    /// Find a document by its short code across all document types
    pub async fn find_document_by_short_code(
        &self,
        short_code: &str,
    ) -> Result<DocumentDiscoveryResult> {
        // Determine document type from short code format (e.g., PROJ-V-0001 -> Vision)
        let doc_type = self.document_type_from_short_code(short_code)?;
        let file_path = self.construct_path_from_short_code(short_code, doc_type)?;

        if file_path.exists() {
            Ok(DocumentDiscoveryResult {
                document_type: doc_type,
                file_path,
            })
        } else {
            Err(MetisError::NotFound(format!(
                "Document with short code '{}' not found at path: {}",
                short_code,
                file_path.display()
            )))
        }
    }

    /// Find a document by its ID across all document types (legacy method)
    pub async fn find_document_by_id(&self, document_id: &str) -> Result<DocumentDiscoveryResult> {
        // Try each document type in order
        for doc_type in [
            DocumentType::Vision,
            DocumentType::Strategy,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ] {
            if let Ok(file_path) = self.find_document_of_type(document_id, doc_type).await {
                return Ok(DocumentDiscoveryResult {
                    document_type: doc_type,
                    file_path,
                });
            }
        }

        Err(MetisError::NotFound(format!(
            "Document '{}' not found in workspace",
            document_id
        )))
    }

    /// Find a document by its ID within a specific document type
    pub async fn find_document_of_type(
        &self,
        document_id: &str,
        doc_type: DocumentType,
    ) -> Result<PathBuf> {
        match doc_type {
            DocumentType::Vision => {
                let file_path = self.workspace_dir.join("vision.md");
                if file_path.exists() {
                    let vision = Vision::from_file(&file_path)
                        .await
                        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                    if vision.id().to_string() == document_id {
                        return Ok(file_path);
                    }
                }
                Err(MetisError::NotFound(
                    "Vision document not found".to_string(),
                ))
            }

            DocumentType::Strategy => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if !strategies_dir.exists() {
                    return Err(MetisError::NotFound(
                        "No strategies directory found".to_string(),
                    ));
                }

                for entry in fs::read_dir(&strategies_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                {
                    let strategy_dir = entry
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        .path();
                    if !strategy_dir.is_dir() {
                        continue;
                    }

                    let file_path = strategy_dir.join("strategy.md");
                    if file_path.exists() {
                        let strategy = Strategy::from_file(&file_path)
                            .await
                            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                        if strategy.id().to_string() == document_id {
                            return Ok(file_path);
                        }
                    }
                }
                Err(MetisError::NotFound(
                    "Strategy document not found".to_string(),
                ))
            }

            DocumentType::Initiative => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if !strategies_dir.exists() {
                    return Err(MetisError::NotFound(
                        "No strategies directory found".to_string(),
                    ));
                }

                for strategy_entry in fs::read_dir(&strategies_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                {
                    let strategy_dir = strategy_entry
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        .path();
                    if !strategy_dir.is_dir() {
                        continue;
                    }

                    let initiatives_dir = strategy_dir.join("initiatives");
                    if !initiatives_dir.exists() {
                        continue;
                    }

                    for initiative_entry in fs::read_dir(&initiatives_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let initiative_dir = initiative_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !initiative_dir.is_dir() {
                            continue;
                        }

                        let file_path = initiative_dir.join("initiative.md");
                        if file_path.exists() {
                            let initiative = Initiative::from_file(&file_path)
                                .await
                                .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                            if initiative.id().to_string() == document_id {
                                return Ok(file_path);
                            }
                        }
                    }
                }

                Err(MetisError::NotFound(
                    "Initiative document not found".to_string(),
                ))
            }

            DocumentType::Task => {
                // First check backlog directory for backlog tasks
                let backlog_dir = self.workspace_dir.join("backlog");
                if backlog_dir.exists() {
                    for entry in fs::read_dir(&backlog_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let task_path = entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if task_path.is_file()
                            && task_path.extension().is_some_and(|ext| ext == "md")
                        {
                            if let Ok(task) = Task::from_file(&task_path).await {
                                if task.id().to_string() == document_id {
                                    return Ok(task_path);
                                }
                            }
                        }
                    }
                }

                // Then check strategies directory for assigned tasks
                let strategies_dir = self.workspace_dir.join("strategies");
                if strategies_dir.exists() {
                    for strategy_entry in fs::read_dir(&strategies_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let strategy_dir = strategy_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !strategy_dir.is_dir() {
                            continue;
                        }

                        let initiatives_dir = strategy_dir.join("initiatives");
                        if !initiatives_dir.exists() {
                            continue;
                        }

                        for initiative_entry in fs::read_dir(&initiatives_dir)
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        {
                            let initiative_dir = initiative_entry
                                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                .path();
                            if !initiative_dir.is_dir() {
                                continue;
                            }

                            // Look for task files in the tasks subdirectory (NULL-based structure)
                            let tasks_dir = initiative_dir.join("tasks");
                            if !tasks_dir.exists() {
                                continue;
                            }

                            for task_entry in fs::read_dir(&tasks_dir)
                                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            {
                                let task_path = task_entry
                                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                    .path();
                                if task_path.is_file()
                                    && task_path.extension().is_some_and(|ext| ext == "md")
                                {
                                    if let Ok(task) = Task::from_file(&task_path).await {
                                        if task.id().to_string() == document_id {
                                            return Ok(task_path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Also check for direct configuration tasks (strategies/NULL/initiatives/NULL/tasks/)
                let direct_tasks_dir = self
                    .workspace_dir
                    .join("strategies")
                    .join("NULL")
                    .join("initiatives")
                    .join("NULL")
                    .join("tasks");

                if direct_tasks_dir.exists() {
                    for task_entry in fs::read_dir(&direct_tasks_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let task_path = task_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if task_path.is_file()
                            && task_path.extension().is_some_and(|ext| ext == "md")
                        {
                            if let Ok(task) = Task::from_file(&task_path).await {
                                if task.id().to_string() == document_id {
                                    return Ok(task_path);
                                }
                            }
                        }
                    }
                }

                Err(MetisError::NotFound("Task document not found".to_string()))
            }

            DocumentType::Adr => {
                let adrs_dir = self.workspace_dir.join("adrs");
                if !adrs_dir.exists() {
                    return Err(MetisError::NotFound("No ADRs directory found".to_string()));
                }

                for entry in
                    fs::read_dir(&adrs_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?
                {
                    let adr_path = entry
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        .path();
                    if adr_path.is_file() && adr_path.extension().is_some_and(|ext| ext == "md") {
                        if let Ok(adr) = Adr::from_file(&adr_path).await {
                            if adr.id().to_string() == document_id {
                                return Ok(adr_path);
                            }
                        }
                    }
                }
                Err(MetisError::NotFound("ADR document not found".to_string()))
            }
        }
    }

    /// Find a document by its ID with a specific document type constraint
    pub async fn find_document_by_id_and_type(
        &self,
        document_id: &str,
        doc_type: DocumentType,
    ) -> Result<PathBuf> {
        self.find_document_of_type(document_id, doc_type).await
    }

    /// Check if a document with the given ID exists
    pub async fn document_exists(&self, document_id: &str) -> bool {
        self.find_document_by_id(document_id).await.is_ok()
    }

    /// Get all documents of a specific type
    pub async fn find_all_documents_of_type(&self, doc_type: DocumentType) -> Result<Vec<PathBuf>> {
        let mut documents = Vec::new();

        match doc_type {
            DocumentType::Vision => {
                let file_path = self.workspace_dir.join("vision.md");
                if file_path.exists() {
                    documents.push(file_path);
                }
            }

            DocumentType::Strategy => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if strategies_dir.exists() {
                    for entry in fs::read_dir(&strategies_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let strategy_dir = entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if strategy_dir.is_dir() {
                            let file_path = strategy_dir.join("strategy.md");
                            if file_path.exists() {
                                documents.push(file_path);
                            }
                        }
                    }
                }
            }

            DocumentType::Initiative => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if strategies_dir.exists() {
                    for strategy_entry in fs::read_dir(&strategies_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let strategy_dir = strategy_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !strategy_dir.is_dir() {
                            continue;
                        }

                        let initiatives_dir = strategy_dir.join("initiatives");
                        if initiatives_dir.exists() {
                            for initiative_entry in fs::read_dir(&initiatives_dir)
                                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            {
                                let initiative_dir = initiative_entry
                                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                    .path();
                                if initiative_dir.is_dir() {
                                    let file_path = initiative_dir.join("initiative.md");
                                    if file_path.exists() {
                                        documents.push(file_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            DocumentType::Task => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if strategies_dir.exists() {
                    for strategy_entry in fs::read_dir(&strategies_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let strategy_dir = strategy_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !strategy_dir.is_dir() {
                            continue;
                        }

                        let initiatives_dir = strategy_dir.join("initiatives");
                        if initiatives_dir.exists() {
                            for initiative_entry in fs::read_dir(&initiatives_dir)
                                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            {
                                let initiative_dir = initiative_entry
                                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                    .path();
                                if !initiative_dir.is_dir() {
                                    continue;
                                }

                                for task_entry in fs::read_dir(&initiative_dir)
                                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                {
                                    let task_path = task_entry
                                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                        .path();
                                    if task_path.is_file()
                                        && task_path.extension().is_some_and(|ext| ext == "md")
                                    {
                                        if task_path
                                            .file_name()
                                            .is_some_and(|name| name == "initiative.md")
                                        {
                                            continue;
                                        }
                                        documents.push(task_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            DocumentType::Adr => {
                let adrs_dir = self.workspace_dir.join("adrs");
                if adrs_dir.exists() {
                    for entry in fs::read_dir(&adrs_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let adr_path = entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if adr_path.is_file() && adr_path.extension().is_some_and(|ext| ext == "md")
                        {
                            documents.push(adr_path);
                        }
                    }
                }
            }
        }

        Ok(documents)
    }

    /// Find all documents in a strategy hierarchy using database lineage queries
    /// This is more efficient than filesystem-based discovery for large hierarchies
    pub async fn find_strategy_hierarchy_with_database(
        &self,
        strategy_id: &str,
        db_service: &mut DatabaseService,
    ) -> Result<Vec<DocumentDiscoveryResult>> {
        let hierarchy_docs = db_service.find_strategy_hierarchy(strategy_id)?;
        let mut results = Vec::new();

        for doc in hierarchy_docs {
            if let Ok(doc_type) = DocumentType::from_str(&doc.document_type) {
                // Convert relative path from DB to absolute path
                let absolute_path = self.workspace_dir.join(&doc.filepath);
                results.push(DocumentDiscoveryResult {
                    document_type: doc_type,
                    file_path: absolute_path,
                });
            }
        }

        Ok(results)
    }

    /// Find all documents in an initiative hierarchy using database lineage queries
    /// This is more efficient than filesystem-based discovery for large hierarchies
    pub async fn find_initiative_hierarchy_with_database(
        &self,
        initiative_id: &str,
        db_service: &mut DatabaseService,
    ) -> Result<Vec<DocumentDiscoveryResult>> {
        let hierarchy_docs = db_service.find_initiative_hierarchy(initiative_id)?;
        let mut results = Vec::new();

        for doc in hierarchy_docs {
            if let Ok(doc_type) = DocumentType::from_str(&doc.document_type) {
                // Convert relative path from DB to absolute path
                let absolute_path = self.workspace_dir.join(&doc.filepath);
                results.push(DocumentDiscoveryResult {
                    document_type: doc_type,
                    file_path: absolute_path,
                });
            }
        }

        Ok(results)
    }

    /// Find all documents belonging to a strategy using database lineage queries
    pub async fn find_documents_by_strategy_with_database(
        &self,
        strategy_id: &str,
        db_service: &mut DatabaseService,
    ) -> Result<Vec<DocumentDiscoveryResult>> {
        let docs = db_service.find_by_strategy_id(strategy_id)?;
        let mut results = Vec::new();

        for doc in docs {
            if let Ok(doc_type) = DocumentType::from_str(&doc.document_type) {
                // Convert relative path from DB to absolute path
                let absolute_path = self.workspace_dir.join(&doc.filepath);
                results.push(DocumentDiscoveryResult {
                    document_type: doc_type,
                    file_path: absolute_path,
                });
            }
        }

        Ok(results)
    }

    /// Find all documents belonging to an initiative using database lineage queries
    pub async fn find_documents_by_initiative_with_database(
        &self,
        initiative_id: &str,
        db_service: &mut DatabaseService,
    ) -> Result<Vec<DocumentDiscoveryResult>> {
        let docs = db_service.find_by_initiative_id(initiative_id)?;
        let mut results = Vec::new();

        for doc in docs {
            if let Ok(doc_type) = DocumentType::from_str(&doc.document_type) {
                // Convert relative path from DB to absolute path
                let absolute_path = self.workspace_dir.join(&doc.filepath);
                results.push(DocumentDiscoveryResult {
                    document_type: doc_type,
                    file_path: absolute_path,
                });
            }
        }

        Ok(results)
    }

    /// Fast document lookup using database instead of filesystem scanning
    /// This is more efficient when the database is synchronized
    pub async fn find_document_by_id_with_database(
        &self,
        document_id: &str,
        db_service: &mut DatabaseService,
    ) -> Result<DocumentDiscoveryResult> {
        let doc = db_service
            .find_by_id(document_id)?
            .ok_or_else(|| MetisError::NotFound(format!("Document '{}' not found", document_id)))?;

        let doc_type = DocumentType::from_str(&doc.document_type).map_err(|e| {
            MetisError::ValidationFailed {
                message: format!("Invalid document type: {}", e),
            }
        })?;

        // Convert relative path from DB to absolute path
        let absolute_path = self.workspace_dir.join(&doc.filepath);

        Ok(DocumentDiscoveryResult {
            document_type: doc_type,
            file_path: absolute_path,
        })
    }

    /// Extract document type from short code format (e.g., PROJ-V-0001 -> Vision)
    fn document_type_from_short_code(&self, short_code: &str) -> Result<DocumentType> {
        let parts: Vec<&str> = short_code.split('-').collect();
        if parts.len() != 3 {
            return Err(MetisError::ValidationFailed {
                message: format!(
                    "Invalid short code format: '{}'. Expected format: PREFIX-TYPE-NNNN",
                    short_code
                ),
            });
        }

        match parts[1] {
            "V" => Ok(DocumentType::Vision),
            "S" => Ok(DocumentType::Strategy),
            "I" => Ok(DocumentType::Initiative),
            "T" => Ok(DocumentType::Task),
            "A" => Ok(DocumentType::Adr),
            _ => Err(MetisError::ValidationFailed {
                message: format!(
                    "Unknown document type code: '{}' in short code '{}'",
                    parts[1], short_code
                ),
            }),
        }
    }

    /// Construct file path from short code and document type
    fn construct_path_from_short_code(
        &self,
        short_code: &str,
        doc_type: DocumentType,
    ) -> Result<PathBuf> {
        match doc_type {
            DocumentType::Vision => Ok(self.workspace_dir.join("vision.md")),
            DocumentType::Strategy => Ok(self
                .workspace_dir
                .join("strategies")
                .join(short_code)
                .join("strategy.md")),
            DocumentType::Initiative => {
                // For initiatives, we need to find via database lookup
                // Fall back to filesystem search if database is not available
                self.find_initiative_path_by_short_code(short_code)
            }
            DocumentType::Task => {
                // For tasks, we need to find via database lookup
                // Fall back to filesystem search if database is not available
                self.find_task_path_by_short_code(short_code)
            }
            DocumentType::Adr => Ok(self
                .workspace_dir
                .join("adrs")
                .join(format!("{}.md", short_code))),
        }
    }

    /// Find initiative path by short code using database lookup
    fn find_initiative_path_by_short_code(&self, short_code: &str) -> Result<PathBuf> {
        // Try database lookup first
        let db_path = self.workspace_dir.join("metis.db");
        if db_path.exists() {
            if let Ok(db) = crate::Database::new(&db_path.to_string_lossy()) {
                if let Ok(mut repo) = db.repository() {
                    if let Ok(Some(doc)) = repo.find_by_short_code(short_code) {
                        if let Some(strategy_id) = &doc.strategy_id {
                            // Find strategy short code from strategy ID
                            if let Ok(Some(strategy_doc)) = repo.find_by_id(strategy_id) {
                                return Ok(self
                                    .workspace_dir
                                    .join("strategies")
                                    .join(&strategy_doc.short_code)
                                    .join("initiatives")
                                    .join(short_code)
                                    .join("initiative.md"));
                            }
                        }
                    }
                }
            }
        }

        // Fall back to filesystem search
        let strategies_dir = self.workspace_dir.join("strategies");
        if !strategies_dir.exists() {
            return Err(MetisError::NotFound(format!(
                "Initiative '{}' not found - no strategies directory",
                short_code
            )));
        }

        for strategy_entry in
            fs::read_dir(&strategies_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?
        {
            let strategy_dir = strategy_entry
                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                .path();
            if !strategy_dir.is_dir() {
                continue;
            }

            let initiative_path = strategy_dir
                .join("initiatives")
                .join(short_code)
                .join("initiative.md");

            if initiative_path.exists() {
                return Ok(initiative_path);
            }
        }

        Err(MetisError::NotFound(format!(
            "Initiative '{}' not found in any strategy",
            short_code
        )))
    }

    /// Find task path by short code using database lookup
    fn find_task_path_by_short_code(&self, short_code: &str) -> Result<PathBuf> {
        // Try database lookup first
        let db_path = self.workspace_dir.join("metis.db");
        if db_path.exists() {
            if let Ok(db) = crate::Database::new(&db_path.to_string_lossy()) {
                if let Ok(mut repo) = db.repository() {
                    if let Ok(Some(doc)) = repo.find_by_short_code(short_code) {
                        // Convert relative path from DB to absolute path
                        return Ok(self.workspace_dir.join(&doc.filepath));
                    }
                }
            }
        }

        // Fall back to filesystem search - check backlog first
        let backlog_path = self
            .workspace_dir
            .join("backlog")
            .join(format!("{}.md", short_code));
        if backlog_path.exists() {
            return Ok(backlog_path);
        }

        // Then check initiative hierarchy
        let strategies_dir = self.workspace_dir.join("strategies");
        if strategies_dir.exists() {
            for strategy_entry in
                fs::read_dir(&strategies_dir).map_err(|e| MetisError::FileSystem(e.to_string()))?
            {
                let strategy_dir = strategy_entry
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    .path();
                if !strategy_dir.is_dir() {
                    continue;
                }

                let initiatives_dir = strategy_dir.join("initiatives");
                if !initiatives_dir.exists() {
                    continue;
                }

                for initiative_entry in fs::read_dir(&initiatives_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                {
                    let initiative_dir = initiative_entry
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        .path();
                    if !initiative_dir.is_dir() {
                        continue;
                    }

                    let task_path = initiative_dir
                        .join("tasks")
                        .join(format!("{}.md", short_code));

                    if task_path.exists() {
                        return Ok(task_path);
                    }
                }
            }
        }

        Err(MetisError::NotFound(format!(
            "Task '{}' not found",
            short_code
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_find_vision_document() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        // Create a simple vision document
        let vision_content = r##"---
id: test-vision
title: Test Vision
level: vision
created_at: 2023-01-01T00:00:00Z
updated_at: 2023-01-01T00:00:00Z
archived: false
short_code: TEST-V-9004
tags:
  - "#vision"
  - "#phase/draft"
exit_criteria_met: false
---

# Test Vision

This is a test vision document.
"##;
        fs::write(workspace_dir.join("vision.md"), vision_content).unwrap();

        let service = DocumentDiscoveryService::new(&workspace_dir);
        let result = service.find_document_by_id("test-vision").await.unwrap();

        assert_eq!(result.document_type, DocumentType::Vision);
        // Canonicalize expected path to match the service's canonical workspace_dir
        let expected_path = workspace_dir.canonicalize().unwrap().join("vision.md");
        assert_eq!(result.file_path, expected_path);
    }

    #[tokio::test]
    async fn test_document_not_found() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        let service = DocumentDiscoveryService::new(&workspace_dir);
        let result = service.find_document_by_id("nonexistent-doc").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MetisError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_find_all_documents_of_type() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        let adrs_dir = workspace_dir.join("adrs");
        fs::create_dir_all(&adrs_dir).unwrap();

        // Create multiple ADR documents
        let adr_content = r##"---
id: test-adr-1
title: Test ADR
level: adr
created_at: 2023-01-01T00:00:00Z
updated_at: 2023-01-01T00:00:00Z
archived: false
number: 1
slug: test-adr
tags:
  - "#adr"
  - "#phase/draft"
exit_criteria_met: false
---

# Test ADR

This is a test ADR document.
"##;
        fs::write(adrs_dir.join("001-test-adr.md"), adr_content).unwrap();
        fs::write(
            adrs_dir.join("002-another-adr.md"),
            adr_content.replace("test-adr-1", "test-adr-2"),
        )
        .unwrap();

        let service = DocumentDiscoveryService::new(&workspace_dir);
        let documents = service
            .find_all_documents_of_type(DocumentType::Adr)
            .await
            .unwrap();

        assert_eq!(documents.len(), 2);
    }
}
