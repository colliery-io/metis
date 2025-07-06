use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use super::metadata::DocumentMetadata;
use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use std::path::Path;
use gray_matter;
use chrono::{DateTime, Utc};
use tera::{Tera, Context};

/// A Task document represents a concrete, actionable piece of work
#[derive(Debug)]
pub struct Task {
    core: super::traits::DocumentCore,
}

impl Task {
    /// Create a new Task document from parsed file data
    pub fn new(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>, // Usually an Initiative
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
    ) -> Self {
        Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
            },
        }
    }

    /// Create a Task document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e)))?;

        Self::from_content(&raw_content)
    }

    /// Create a Task document from raw file content string
    pub fn from_content(raw_content: &str) -> Result<Self, DocumentValidationError> {
        // Parse frontmatter and content
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(raw_content);
        
        // Extract frontmatter data
        let frontmatter = parsed.data.ok_or_else(|| {
            DocumentValidationError::MissingRequiredField("frontmatter".to_string())
        })?;

        // Parse frontmatter into structured data
        let fm_map = match frontmatter {
            gray_matter::Pod::Hash(map) => map,
            _ => return Err(DocumentValidationError::InvalidContent("Frontmatter must be a hash/map".to_string())),
        };

        // Extract required fields
        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);

        // Parse timestamps
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met = FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);

        // Parse tags
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        // Verify this is actually a task document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "task" {
            return Err(DocumentValidationError::InvalidContent(
                format!("Expected level 'task', found '{}'", level)
            ));
        }

        // Extract task-specific fields
        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent").ok().map(DocumentId::from);
        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();

        // Create metadata and content
        let metadata = DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::new(title, metadata, content, parent_id, blocked_by, tags, archived))
    }

    /// Write the Task document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e)))
    }

    /// Convert the Task document to its markdown string representation using templates
    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();
        
        // Add the templates to Tera
        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Template error: {}", e)))?;
        tera.add_raw_template("content", self.content_template())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Template error: {}", e)))?;
        tera.add_raw_template("acceptance_criteria", self.acceptance_criteria_template())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Template error: {}", e)))?;
        
        // Create context with all document data
        let mut context = Context::new();
        context.insert("slug", &self.id().to_string());
        context.insert("title", self.title());
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived());
        context.insert("exit_criteria_met", &self.metadata().exit_criteria_met);
        context.insert("parent_id", &self.parent_id().map(|id| id.to_string()).unwrap_or_default());
        context.insert("blocked_by", &self.blocked_by().iter().map(|id| id.to_string()).collect::<Vec<_>>());
        
        // Convert tags to strings
        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);
        
        // Render frontmatter
        let frontmatter = tera.render("frontmatter", &context)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e)))?;
        
        // Add content body and acceptance criteria to context
        context.insert("body", &self.content().body);
        context.insert("acceptance_criteria_content", &self.content().acceptance_criteria.as_deref().unwrap_or(""));
        
        // Render content
        let content_body = tera.render("content", &context)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Content render error: {}", e)))?;
        
        // Render acceptance criteria
        let acceptance_criteria = tera.render("acceptance_criteria", &context)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Acceptance criteria render error: {}", e)))?;
        
        // Combine everything
        Ok(format!("---\n{}---\n\n{}\n\n{}", frontmatter, content_body, acceptance_criteria))
    }

}

impl Document for Task {
    // id() uses default implementation from trait

    fn document_type(&self) -> DocumentType {
        DocumentType::Task
    }

    fn title(&self) -> &str {
        &self.core.title
    }

    fn metadata(&self) -> &DocumentMetadata {
        &self.core.metadata
    }

    fn content(&self) -> &DocumentContent {
        &self.core.content
    }

    fn core(&self) -> &super::traits::DocumentCore {
        &self.core
    }

    fn can_transition_to(&self, phase: Phase) -> bool {
        if let Ok(current_phase) = self.phase() {
            use Phase::*;
            match (current_phase, phase) {
                (Todo, Active) => true,
                (Active, Completed) => true,
                (Active, Blocked) => true,
                (Todo, Blocked) => true, // Can pre emptively be blocked while in backlog
                (Blocked, Active) => true,
                (Blocked, Todo) => true, // Can go back to todo if unblocked
                _ => false,
            }
        } else {
            false // Can't transition if we can't determine current phase
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        self.core.parent_id.as_ref()
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &self.core.blocked_by
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        // Task-specific validation rules
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "Task title cannot be empty".to_string(),
            ));
        }

        // Tasks should typically have a parent (Initiative)
        if self.parent_id().is_none() {
            return Err(DocumentValidationError::MissingRequiredField(
                "Tasks should have a parent Initiative".to_string(),
            ));
        }

        // If blocked, must have blocking documents listed
        if let Ok(Phase::Blocked) = self.phase() {
            if self.blocked_by().is_empty() {
                return Err(DocumentValidationError::InvalidContent(
                    "Blocked tasks must specify what they are blocked by".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn exit_criteria_met(&self) -> bool {
        // Check if all acceptance criteria checkboxes are checked
        // This would typically parse the content for checkbox completion
        // For now, return false as a placeholder
        false
    }

    fn template(&self) -> DocumentTemplate {
        DocumentTemplate {
            frontmatter: self.frontmatter_template(),
            content: self.content_template(),
            acceptance_criteria: self.acceptance_criteria_template(),
            file_extension: "md",
        }
    }

    fn frontmatter_template(&self) -> &'static str {
        include_str!("frontmatter.yaml")
    }

    fn content_template(&self) -> &'static str {
        include_str!("content.md")
    }

    fn acceptance_criteria_template(&self) -> &'static str {
        include_str!("acceptance_criteria.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_from_content() {
        let content = r##"---
id: test-task
level: task
title: "Test Task"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
parent: initiative-001
blocked_by: []

tags:
  - "#task"
  - "#phase/todo"

exit_criteria_met: false
---

# Test Task

## Description

This is a test task for our system.

## Implementation Notes

Details on how to implement this.

## Acceptance Criteria

- [ ] Implementation is complete
- [ ] Tests pass
"##;

        let task = Task::from_content(content).unwrap();
        
        assert_eq!(task.title(), "Test Task");
        assert_eq!(task.document_type(), DocumentType::Task);
        assert!(!task.archived());
        assert_eq!(task.tags().len(), 2);
        assert_eq!(task.phase().unwrap(), Phase::Todo);
        assert!(task.content().has_acceptance_criteria());
    }


    #[test]
    fn test_task_invalid_level() {
        let content = r##"---
id: test-doc
level: strategy
title: "Test Strategy"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
tags:
  - "#strategy"
  - "#phase/shaping"
exit_criteria_met: false
---

# Test Strategy
"##;

        let result = Task::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidContent(msg) => {
                assert!(msg.contains("Expected level 'task'"));
            }
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_task_validation() {
        let task = Task::new(
            "Test Task".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            Some(DocumentId::from("parent-initiative")),
            vec![],
            vec![Tag::Label("task".to_string()), Tag::Phase(Phase::Todo)],
            false,
        );

        assert!(task.validate().is_ok());
        
        // Test validation failure - no parent
        let task_no_parent = Task::new(
            "Test Task".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None, // No parent
            vec![],
            vec![Tag::Phase(Phase::Todo)],
            false,
        );
        
        assert!(task_no_parent.validate().is_err());
    }

    #[test]
    fn test_task_blocked_validation() {
        // Task marked as blocked but no blocking documents
        let blocked_task = Task::new(
            "Blocked Task".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            Some(DocumentId::from("parent-initiative")),
            vec![], // No blocking documents
            vec![Tag::Phase(Phase::Blocked)],
            false,
        );
        
        assert!(blocked_task.validate().is_err());
        
        // Task marked as blocked with blocking documents
        let properly_blocked_task = Task::new(
            "Blocked Task".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            Some(DocumentId::from("parent-initiative")),
            vec![DocumentId::from("blocking-task")],
            vec![Tag::Phase(Phase::Blocked)],
            false,
        );
        
        assert!(properly_blocked_task.validate().is_ok());
    }

    #[test]
    fn test_task_phase_transitions() {
        let task = Task::new(
            "Test Task".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            Some(DocumentId::from("parent-initiative")),
            vec![],
            vec![Tag::Phase(Phase::Todo)],
            false,
        );

        assert!(task.can_transition_to(Phase::Active));
        assert!(task.can_transition_to(Phase::Blocked));
        assert!(!task.can_transition_to(Phase::Completed));
        assert!(!task.can_transition_to(Phase::Design));
    }

    #[test]
    fn test_task_active_phase_transitions() {
        let active_task = Task::new(
            "Active Task".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            Some(DocumentId::from("parent-initiative")),
            vec![],
            vec![Tag::Phase(Phase::Active)],
            false,
        );

        assert!(active_task.can_transition_to(Phase::Completed));
        assert!(active_task.can_transition_to(Phase::Blocked));
        assert!(!active_task.can_transition_to(Phase::Todo));
    }

    #[test]
    fn test_task_blocked_phase_transitions() {
        let blocked_task = Task::new(
            "Blocked Task".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            Some(DocumentId::from("parent-initiative")),
            vec![DocumentId::from("blocking-task")],
            vec![Tag::Phase(Phase::Blocked)],
            false,
        );

        assert!(blocked_task.can_transition_to(Phase::Active));
        assert!(blocked_task.can_transition_to(Phase::Todo));
        assert!(!blocked_task.can_transition_to(Phase::Completed));
    }
}