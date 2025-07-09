use crate::domain::documents::types::DocumentType;
use crate::domain::documents::traits::Document;
use crate::{Vision, Strategy, Initiative, Task, Adr, MetisError};
use crate::Result;
use std::path::{Path, PathBuf};
use std::fs;

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
        Self {
            workspace_dir: workspace_dir.as_ref().to_path_buf(),
        }
    }

    /// Find a document by its ID across all document types
    pub async fn find_document_by_id(&self, document_id: &str) -> Result<DocumentDiscoveryResult> {
        // Try each document type in order
        for doc_type in [DocumentType::Vision, DocumentType::Strategy, DocumentType::Initiative, DocumentType::Task, DocumentType::Adr] {
            if let Ok(file_path) = self.find_document_of_type(document_id, doc_type).await {
                return Ok(DocumentDiscoveryResult {
                    document_type: doc_type,
                    file_path,
                });
            }
        }
        
        Err(MetisError::NotFound(format!("Document '{}' not found in workspace", document_id)))
    }

    /// Find a document by its ID within a specific document type
    pub async fn find_document_of_type(&self, document_id: &str, doc_type: DocumentType) -> Result<PathBuf> {
        match doc_type {
            DocumentType::Vision => {
                let file_path = self.workspace_dir.join("vision.md");
                if file_path.exists() {
                    let vision = Vision::from_file(&file_path).await
                        .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                    if vision.id().to_string() == document_id {
                        return Ok(file_path);
                    }
                }
                Err(MetisError::NotFound("Vision document not found".to_string()))
            },
            
            DocumentType::Strategy => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if !strategies_dir.exists() {
                    return Err(MetisError::NotFound("No strategies directory found".to_string()));
                }
                
                for entry in fs::read_dir(&strategies_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                    let strategy_dir = entry
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        .path();
                    if !strategy_dir.is_dir() {
                        continue;
                    }
                    
                    let file_path = strategy_dir.join("strategy.md");
                    if file_path.exists() {
                        let strategy = Strategy::from_file(&file_path).await
                            .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                        if strategy.id().to_string() == document_id {
                            return Ok(file_path);
                        }
                    }
                }
                Err(MetisError::NotFound("Strategy document not found".to_string()))
            },
            
            DocumentType::Initiative => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if !strategies_dir.exists() {
                    return Err(MetisError::NotFound("No strategies directory found".to_string()));
                }
                
                for strategy_entry in fs::read_dir(&strategies_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))? {
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
                        .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                        let initiative_dir = initiative_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !initiative_dir.is_dir() {
                            continue;
                        }
                        
                        let file_path = initiative_dir.join("initiative.md");
                        if file_path.exists() {
                            let initiative = Initiative::from_file(&file_path).await
                                .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                            if initiative.id().to_string() == document_id {
                                return Ok(file_path);
                            }
                        }
                    }
                }
                Err(MetisError::NotFound("Initiative document not found".to_string()))
            },
            
            DocumentType::Task => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if !strategies_dir.exists() {
                    return Err(MetisError::NotFound("No strategies directory found".to_string()));
                }
                
                for strategy_entry in fs::read_dir(&strategies_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))? {
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
                        .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                        let initiative_dir = initiative_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !initiative_dir.is_dir() {
                            continue;
                        }
                        
                        // Look for task files in the initiative directory
                        for task_entry in fs::read_dir(&initiative_dir)
                            .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                            let task_path = task_entry
                                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                .path();
                            if task_path.is_file() && task_path.extension().map_or(false, |ext| ext == "md") {
                                // Skip initiative.md
                                if task_path.file_name().map_or(false, |name| name == "initiative.md") {
                                    continue;
                                }
                                
                                if let Ok(task) = Task::from_file(&task_path).await {
                                    if task.id().to_string() == document_id {
                                        return Ok(task_path);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(MetisError::NotFound("Task document not found".to_string()))
            },
            
            DocumentType::Adr => {
                let adrs_dir = self.workspace_dir.join("adrs");
                if !adrs_dir.exists() {
                    return Err(MetisError::NotFound("No ADRs directory found".to_string()));
                }
                
                for entry in fs::read_dir(&adrs_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                    let adr_path = entry
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        .path();
                    if adr_path.is_file() && adr_path.extension().map_or(false, |ext| ext == "md") {
                        if let Ok(adr) = Adr::from_file(&adr_path).await {
                            if adr.id().to_string() == document_id {
                                return Ok(adr_path);
                            }
                        }
                    }
                }
                Err(MetisError::NotFound("ADR document not found".to_string()))
            },
        }
    }

    /// Find a document by its ID with a specific document type constraint
    pub async fn find_document_by_id_and_type(&self, document_id: &str, doc_type: DocumentType) -> Result<PathBuf> {
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
            },
            
            DocumentType::Strategy => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if strategies_dir.exists() {
                    for entry in fs::read_dir(&strategies_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))? {
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
            },
            
            DocumentType::Initiative => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if strategies_dir.exists() {
                    for strategy_entry in fs::read_dir(&strategies_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                        let strategy_dir = strategy_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !strategy_dir.is_dir() {
                            continue;
                        }
                        
                        let initiatives_dir = strategy_dir.join("initiatives");
                        if initiatives_dir.exists() {
                            for initiative_entry in fs::read_dir(&initiatives_dir)
                                .map_err(|e| MetisError::FileSystem(e.to_string()))? {
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
            },
            
            DocumentType::Task => {
                let strategies_dir = self.workspace_dir.join("strategies");
                if strategies_dir.exists() {
                    for strategy_entry in fs::read_dir(&strategies_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                        let strategy_dir = strategy_entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if !strategy_dir.is_dir() {
                            continue;
                        }
                        
                        let initiatives_dir = strategy_dir.join("initiatives");
                        if initiatives_dir.exists() {
                            for initiative_entry in fs::read_dir(&initiatives_dir)
                                .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                                let initiative_dir = initiative_entry
                                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                    .path();
                                if !initiative_dir.is_dir() {
                                    continue;
                                }
                                
                                for task_entry in fs::read_dir(&initiative_dir)
                                    .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                                    let task_path = task_entry
                                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                                        .path();
                                    if task_path.is_file() && task_path.extension().map_or(false, |ext| ext == "md") {
                                        if task_path.file_name().map_or(false, |name| name == "initiative.md") {
                                            continue;
                                        }
                                        documents.push(task_path);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            
            DocumentType::Adr => {
                let adrs_dir = self.workspace_dir.join("adrs");
                if adrs_dir.exists() {
                    for entry in fs::read_dir(&adrs_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))? {
                        let adr_path = entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if adr_path.is_file() && adr_path.extension().map_or(false, |ext| ext == "md") {
                            documents.push(adr_path);
                        }
                    }
                }
            },
        }
        
        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

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
        assert_eq!(result.file_path, workspace_dir.join("vision.md"));
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
        fs::write(adrs_dir.join("002-another-adr.md"), adr_content.replace("test-adr-1", "test-adr-2")).unwrap();

        let service = DocumentDiscoveryService::new(&workspace_dir);
        let documents = service.find_all_documents_of_type(DocumentType::Adr).await.unwrap();
        
        assert_eq!(documents.len(), 2);
    }
}