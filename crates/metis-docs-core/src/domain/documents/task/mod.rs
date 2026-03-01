use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// A Task document represents a concrete, actionable piece of work
#[derive(Debug)]
pub struct Task {
    core: super::traits::DocumentCore,
}

impl Task {
    /// Create a new Task document with content rendered from template
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>,     // Usually an Initiative
        parent_title: Option<String>,      // Title of parent for template rendering
        strategy_id: Option<DocumentId>,   // The strategy this task belongs to
        initiative_id: Option<DocumentId>, // The initiative this task belongs to
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
    ) -> Result<Self, DocumentValidationError> {
        // Use embedded default template
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            parent_title,
            strategy_id,
            initiative_id,
            blocked_by,
            tags,
            archived,
            short_code,
            template_content,
        )
    }

    /// Create a new Task document with a custom template
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_template(
        title: String,
        parent_id: Option<DocumentId>,
        parent_title: Option<String>,
        strategy_id: Option<DocumentId>,
        initiative_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        // Create fresh metadata
        let metadata = DocumentMetadata::new(short_code);

        // Render the content template
        let mut tera = Tera::default();
        tera.add_raw_template("task_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);
        context.insert(
            "parent_title",
            &parent_title.unwrap_or_else(|| "Parent Initiative".to_string()),
        );

        let rendered_content = tera.render("task_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
                strategy_id,
                initiative_id,
            },
        })
    }

    /// Create a Task document from existing data (used when loading from file)
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        strategy_id: Option<DocumentId>,
        initiative_id: Option<DocumentId>,
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
                strategy_id,
                initiative_id,
            },
        }
    }

    /// Create a Task document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

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
            _ => {
                return Err(DocumentValidationError::InvalidContent(
                    "Frontmatter must be a hash/map".to_string(),
                ))
            }
        };

        // Extract required fields
        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);

        // Parse timestamps
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);

        // Parse tags
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        // Verify this is actually a task document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "task" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'task', found '{}'",
                level
            )));
        }

        // Extract task-specific fields
        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent")
            .ok()
            .map(DocumentId::from);
        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();

        // Create metadata and content
        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        // Extract lineage from frontmatter
        let strategy_id = FrontmatterParser::extract_optional_string(&fm_map, "strategy_id")
            .map(DocumentId::from);
        let initiative_id = FrontmatterParser::extract_optional_string(&fm_map, "initiative_id")
            .map(DocumentId::from);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            parent_id,
            strategy_id,
            initiative_id,
            blocked_by,
            tags,
            archived,
        ))
    }

    /// Get the next phase in the Task sequence
    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Backlog => None, // Backlog doesn't auto-transition - must be explicitly assigned
            Todo => Some(Active),
            Active => Some(Completed),
            Completed => None, // Final phase
            Blocked => None,   // Blocked doesn't auto-transition
            _ => None,         // Invalid phase for Task
        }
    }

    /// Update the phase tag in the document's tags
    fn update_phase_tag(&mut self, new_phase: Phase) {
        // Remove any existing phase tags
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        // Add the new phase tag
        self.core.tags.push(Tag::Phase(new_phase));
        // Update timestamp
        self.core.metadata.updated_at = Utc::now();
    }

    /// Write the Task document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    /// Convert the Task document to its markdown string representation using templates
    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();

        // Add the frontmatter template to Tera
        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        // Create context with all document data
        let mut context = Context::new();
        context.insert("slug", &self.id().to_string());
        context.insert("title", self.title());
        context.insert("short_code", &self.metadata().short_code);
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.metadata().exit_criteria_met.to_string(),
        );
        context.insert(
            "parent_id",
            &self
                .parent_id()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        );
        let blocked_by_list: Vec<String> =
            self.blocked_by().iter().map(|id| id.to_string()).collect();
        context.insert("blocked_by", &blocked_by_list);

        // Convert tags to strings
        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        // Add lineage fields
        context.insert(
            "strategy_id",
            &self
                .core
                .strategy_id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
        );
        context.insert(
            "initiative_id",
            &self
                .core
                .initiative_id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "NULL".to_string()),
        );

        // Render frontmatter
        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        // Use the actual content body
        let content_body = &self.content().body;

        // Use actual acceptance criteria if present, otherwise empty string
        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{}", ac)
        } else {
            String::new()
        };

        // Combine everything
        Ok(format!(
            "---\n{}\n---\n\n{}{}",
            frontmatter.trim_end(),
            content_body,
            acceptance_criteria
        ))
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
            // Delegate to DocumentType - the single source of truth
            DocumentType::Task.can_transition(current_phase, phase)
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

        // Tasks should have a parent (Initiative) unless they are in Backlog phase
        if self.parent_id().is_none() {
            // Allow no parent only if task is in Backlog phase
            if let Ok(phase) = self.phase() {
                if phase != Phase::Backlog {
                    return Err(DocumentValidationError::MissingRequiredField(
                        "Tasks should have a parent Initiative unless in Backlog phase".to_string(),
                    ));
                }
            } else {
                return Err(DocumentValidationError::MissingRequiredField(
                    "Tasks should have a parent Initiative".to_string(),
                ));
            }
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

    fn transition_phase(
        &mut self,
        target_phase: Option<Phase>,
    ) -> Result<Phase, DocumentValidationError> {
        let current_phase = self.phase()?;

        let new_phase = match target_phase {
            Some(phase) => {
                // Validate the transition is allowed
                if !self.can_transition_to(phase) {
                    return Err(DocumentValidationError::InvalidPhaseTransition {
                        from: current_phase,
                        to: phase,
                    });
                }
                phase
            }
            None => {
                // Auto-transition to next phase in sequence
                match Self::next_phase_in_sequence(current_phase) {
                    Some(next) => next,
                    None => return Ok(current_phase), // Already at final phase or blocked
                }
            }
        };

        self.update_phase_tag(new_phase);
        Ok(new_phase)
    }

    fn core_mut(&mut self) -> &mut super::traits::DocumentCore {
        &mut self.core
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::traits::DocumentValidationError;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_task_from_content() {
        let content = r##"---
id: test-task
level: task
title: "Test Task"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
parent: initiative-001
blocked_by: []
short_code: TEST-T-9001

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

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-task.md");

        task.to_file(&file_path).await.unwrap();
        let loaded_task = Task::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_task.title(), task.title());
        assert_eq!(loaded_task.phase().unwrap(), task.phase().unwrap());
        assert_eq!(loaded_task.content().body, task.content().body);
        assert_eq!(loaded_task.archived(), task.archived());
        assert_eq!(loaded_task.tags().len(), task.tags().len());
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
  - "#phase/draft"
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
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![],                                      // blocked_by
            vec![Tag::Label("task".to_string()), Tag::Phase(Phase::Todo)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        assert!(task.validate().is_ok());

        // Test validation failure - no parent
        let task_no_parent = Task::new(
            "Test Task".to_string(),
            None,   // No parent
            None,   // No parent title
            None,   // No strategy
            None,   // No initiative
            vec![], // blocked_by
            vec![Tag::Phase(Phase::Todo)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        assert!(task_no_parent.validate().is_err());
    }

    #[test]
    fn test_task_blocked_validation() {
        // Task marked as blocked but no blocking documents
        let blocked_task = Task::new(
            "Blocked Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![],                                      // No blocking documents
            vec![Tag::Phase(Phase::Blocked)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        assert!(blocked_task.validate().is_err());

        // Task marked as blocked with blocking documents
        let properly_blocked_task = Task::new(
            "Blocked Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![DocumentId::from("blocking-task")],
            vec![Tag::Phase(Phase::Blocked)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        assert!(properly_blocked_task.validate().is_ok());
    }

    #[test]
    fn test_task_phase_transitions() {
        let task = Task::new(
            "Test Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![],
            vec![Tag::Phase(Phase::Todo)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        assert!(task.can_transition_to(Phase::Active));
        assert!(task.can_transition_to(Phase::Blocked));
        assert!(!task.can_transition_to(Phase::Completed));
        assert!(!task.can_transition_to(Phase::Design));
    }

    #[test]
    fn test_task_active_phase_transitions() {
        let active_task = Task::new(
            "Active Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![],
            vec![Tag::Phase(Phase::Active)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        assert!(active_task.can_transition_to(Phase::Completed));
        assert!(active_task.can_transition_to(Phase::Blocked));
        assert!(!active_task.can_transition_to(Phase::Todo));
    }

    #[test]
    fn test_task_blocked_phase_transitions() {
        let blocked_task = Task::new(
            "Blocked Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![DocumentId::from("blocking-task")],
            vec![Tag::Phase(Phase::Blocked)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        assert!(blocked_task.can_transition_to(Phase::Active));
        assert!(blocked_task.can_transition_to(Phase::Todo));
        assert!(!blocked_task.can_transition_to(Phase::Completed));
    }

    #[test]
    fn test_task_transition_phase_auto() {
        let mut task = Task::new(
            "Test Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![],
            vec![Tag::Phase(Phase::Todo)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        // Auto-transition from Todo should go to Active
        let new_phase = task.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Active);
        assert_eq!(task.phase().unwrap(), Phase::Active);

        // Auto-transition from Active should go to Completed
        let new_phase = task.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Completed);
        assert_eq!(task.phase().unwrap(), Phase::Completed);

        // Auto-transition from Completed should stay at Completed (final phase)
        let new_phase = task.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Completed);
        assert_eq!(task.phase().unwrap(), Phase::Completed);
    }

    #[test]
    fn test_task_transition_phase_blocking() {
        let mut task = Task::new(
            "Test Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![DocumentId::from("blocking-task")],
            vec![Tag::Phase(Phase::Todo)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        // Explicit transition from Todo to Blocked
        let new_phase = task.transition_phase(Some(Phase::Blocked)).unwrap();
        assert_eq!(new_phase, Phase::Blocked);
        assert_eq!(task.phase().unwrap(), Phase::Blocked);

        // Transition from Blocked back to Active (unblocking)
        let new_phase = task.transition_phase(Some(Phase::Active)).unwrap();
        assert_eq!(new_phase, Phase::Active);
        assert_eq!(task.phase().unwrap(), Phase::Active);

        // Blocked doesn't auto-transition
        task.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        task.core.tags.push(Tag::Phase(Phase::Blocked));
        let new_phase = task.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Blocked); // Should stay blocked
    }

    #[test]
    fn test_task_transition_phase_invalid() {
        let mut task = Task::new(
            "Test Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![],
            vec![Tag::Phase(Phase::Todo)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        // Invalid transition from Todo to Completed (must go through Active)
        let result = task.transition_phase(Some(Phase::Completed));
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidPhaseTransition { from, to } => {
                assert_eq!(from, Phase::Todo);
                assert_eq!(to, Phase::Completed);
            }
            _ => panic!("Expected InvalidPhaseTransition error"),
        }

        // Should still be in Todo phase
        assert_eq!(task.phase().unwrap(), Phase::Todo);
    }

    #[test]
    fn test_task_update_section() {
        // First create a task with the template
        let mut task = Task::new(
            "Test Task".to_string(),
            Some(DocumentId::from("parent-initiative")), // parent_id
            Some("Parent Initiative".to_string()),       // parent_title
            Some(DocumentId::from("parent-strategy")),   // strategy_id
            Some(DocumentId::from("parent-initiative")), // initiative_id
            vec![],
            vec![Tag::Phase(Phase::Todo)],
            false,
            "TEST-T-0401".to_string(),
        )
        .expect("Failed to create task");

        // Then update its content to have specific test content
        task.core_mut().content = DocumentContent::new(
            "## Description\n\nOriginal description\n\n## Implementation Notes\n\nOriginal notes",
        );

        // Replace existing section
        task.update_section("Updated task description", "Description", false)
            .unwrap();
        let content = task.content().body.clone();
        assert!(content.contains("## Description\n\nUpdated task description"));
        assert!(!content.contains("Original description"));

        // Append to existing section
        task.update_section(
            "Additional implementation details",
            "Implementation Notes",
            true,
        )
        .unwrap();
        let content = task.content().body.clone();
        assert!(content.contains("Original notes"));
        assert!(content.contains("Additional implementation details"));

        // Add new section
        task.update_section("Test approach details", "Testing Strategy", false)
            .unwrap();
        let content = task.content().body.clone();
        assert!(content.contains("## Testing Strategy\n\nTest approach details"));
    }
}
