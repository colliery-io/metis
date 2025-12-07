//! Template loading service with fallback chain support.
//!
//! This service loads document templates from:
//! 1. Project-level: `.metis/templates/{type}/content.md`
//! 2. Global-level: `~/.config/metis/templates/{type}/content.md`
//! 3. Embedded defaults (compiled into binary)

use std::path::{Path, PathBuf};
use tera::{Context, Tera};

/// Embedded default templates for each document type
mod defaults {
    pub mod vision {
        pub const CONTENT: &str = include_str!("../../domain/documents/vision/content.md");
        pub const EXIT_CRITERIA: &str =
            include_str!("../../domain/documents/vision/acceptance_criteria.md");
    }

    pub mod strategy {
        pub const CONTENT: &str = include_str!("../../domain/documents/strategy/content.md");
        pub const EXIT_CRITERIA: &str =
            include_str!("../../domain/documents/strategy/acceptance_criteria.md");
    }

    pub mod initiative {
        pub const CONTENT: &str = include_str!("../../domain/documents/initiative/content.md");
        pub const EXIT_CRITERIA: &str =
            include_str!("../../domain/documents/initiative/acceptance_criteria.md");
    }

    pub mod task {
        pub const CONTENT: &str = include_str!("../../domain/documents/task/content.md");
        pub const EXIT_CRITERIA: &str =
            include_str!("../../domain/documents/task/acceptance_criteria.md");
    }

    pub mod adr {
        pub const CONTENT: &str = include_str!("../../domain/documents/adr/content.md");
        pub const EXIT_CRITERIA: &str =
            include_str!("../../domain/documents/adr/acceptance_criteria.md");
    }
}

/// Error type for template loading operations
#[derive(Debug, Clone)]
pub enum TemplateError {
    /// Template file could not be read
    IoError(String),
    /// Template failed to parse as valid Tera template
    ParseError(String),
    /// Template failed validation (render with sample data)
    ValidationError(String),
    /// Unknown document type
    UnknownDocumentType(String),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::IoError(msg) => write!(f, "Template IO error: {}", msg),
            TemplateError::ParseError(msg) => write!(f, "Template parse error: {}", msg),
            TemplateError::ValidationError(msg) => write!(f, "Template validation error: {}", msg),
            TemplateError::UnknownDocumentType(t) => write!(f, "Unknown document type: {}", t),
        }
    }
}

impl std::error::Error for TemplateError {}

/// Template types that can be loaded
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    Content,
    ExitCriteria,
}

impl TemplateType {
    fn filename(&self) -> &'static str {
        match self {
            TemplateType::Content => "content.md",
            TemplateType::ExitCriteria => "exit_criteria.md",
        }
    }
}

/// Service for loading templates with fallback chain support.
///
/// Templates are loaded in this order:
/// 1. Project-level: `{workspace}/.metis/templates/{type}/{template}.md`
/// 2. Global-level: `~/.config/metis/templates/{type}/{template}.md`
/// 3. Embedded defaults
pub struct TemplateLoader {
    /// Path to the project workspace (e.g., `/path/to/project/.metis`)
    project_path: Option<PathBuf>,
    /// Path to global config (e.g., `~/.config/metis`)
    global_path: PathBuf,
}

impl TemplateLoader {
    /// Create a new TemplateLoader with the given project workspace path.
    ///
    /// If `project_path` is None, only global and embedded templates will be used.
    pub fn new(project_path: Option<PathBuf>) -> Self {
        let global_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("metis");

        Self {
            project_path,
            global_path,
        }
    }

    /// Create a TemplateLoader for a specific workspace directory.
    pub fn for_workspace<P: AsRef<Path>>(workspace_dir: P) -> Self {
        Self::new(Some(workspace_dir.as_ref().to_path_buf()))
    }

    /// Load a content template for the given document type.
    ///
    /// Returns the template content string, loading from the first available source
    /// in the fallback chain.
    pub fn load_content_template(&self, doc_type: &str) -> Result<String, TemplateError> {
        self.load_template(doc_type, TemplateType::Content)
    }

    /// Load an exit criteria template for the given document type.
    pub fn load_exit_criteria_template(&self, doc_type: &str) -> Result<String, TemplateError> {
        self.load_template(doc_type, TemplateType::ExitCriteria)
    }

    /// Load a template with the fallback chain.
    fn load_template(
        &self,
        doc_type: &str,
        template_type: TemplateType,
    ) -> Result<String, TemplateError> {
        // 1. Try project-level template
        if let Some(ref project_path) = self.project_path {
            let project_template = project_path
                .join("templates")
                .join(doc_type)
                .join(template_type.filename());

            if project_template.exists() {
                let content = std::fs::read_to_string(&project_template)
                    .map_err(|e| TemplateError::IoError(e.to_string()))?;

                // Validate the template before returning
                self.validate_template(&content, doc_type)?;
                return Ok(content);
            }
        }

        // 2. Try global-level template
        let global_template = self
            .global_path
            .join("templates")
            .join(doc_type)
            .join(template_type.filename());

        if global_template.exists() {
            let content = std::fs::read_to_string(&global_template)
                .map_err(|e| TemplateError::IoError(e.to_string()))?;

            // Validate the template before returning
            self.validate_template(&content, doc_type)?;
            return Ok(content);
        }

        // 3. Fall back to embedded defaults
        self.get_embedded_template(doc_type, template_type)
    }

    /// Get the embedded default template for a document type.
    fn get_embedded_template(
        &self,
        doc_type: &str,
        template_type: TemplateType,
    ) -> Result<String, TemplateError> {
        let template = match (doc_type, template_type) {
            ("vision", TemplateType::Content) => defaults::vision::CONTENT,
            ("vision", TemplateType::ExitCriteria) => defaults::vision::EXIT_CRITERIA,
            ("strategy", TemplateType::Content) => defaults::strategy::CONTENT,
            ("strategy", TemplateType::ExitCriteria) => defaults::strategy::EXIT_CRITERIA,
            ("initiative", TemplateType::Content) => defaults::initiative::CONTENT,
            ("initiative", TemplateType::ExitCriteria) => defaults::initiative::EXIT_CRITERIA,
            ("task", TemplateType::Content) => defaults::task::CONTENT,
            ("task", TemplateType::ExitCriteria) => defaults::task::EXIT_CRITERIA,
            ("adr", TemplateType::Content) => defaults::adr::CONTENT,
            ("adr", TemplateType::ExitCriteria) => defaults::adr::EXIT_CRITERIA,
            _ => return Err(TemplateError::UnknownDocumentType(doc_type.to_string())),
        };

        Ok(template.to_string())
    }

    /// Validate a template by rendering it with sample data.
    ///
    /// This catches template syntax errors and missing variable references early.
    pub fn validate_template(&self, template: &str, doc_type: &str) -> Result<(), TemplateError> {
        let mut tera = Tera::default();

        // Try to parse the template
        tera.add_raw_template("test_template", template)
            .map_err(|e| TemplateError::ParseError(e.to_string()))?;

        // Try to render with sample context
        let context = self.sample_context_for_type(doc_type);
        tera.render("test_template", &context)
            .map_err(|e| TemplateError::ValidationError(e.to_string()))?;

        Ok(())
    }

    /// Generate sample context values for validating templates.
    ///
    /// Each document type gets appropriate sample values for all available variables.
    pub fn sample_context_for_type(&self, doc_type: &str) -> Context {
        let mut context = Context::new();

        // Common variables for all document types
        context.insert("title", "Sample Document Title");
        context.insert("slug", "sample-document-title");
        context.insert("short_code", &format!("TEST-{}-0001", doc_type_letter(doc_type)));
        context.insert("created_at", "2025-01-01T00:00:00Z");
        context.insert("updated_at", "2025-01-01T00:00:00Z");
        context.insert("archived", "false");
        context.insert("exit_criteria_met", "false");
        context.insert("parent_id", "");
        context.insert("parent_title", "");
        context.insert("blocked_by", &Vec::<String>::new());
        context.insert("tags", &vec!["#sample", "#phase/draft"]);

        // Type-specific variables
        match doc_type {
            "vision" => {
                // Vision has no additional required variables
            }
            "strategy" => {
                context.insert("risk_level", "Medium");
                context.insert("stakeholders", &Vec::<String>::new());
            }
            "initiative" => {
                context.insert("estimated_complexity", "M");
                context.insert("strategy_id", "NULL");
                context.insert("initiative_id", "sample-initiative");
            }
            "task" => {
                context.insert("strategy_id", "NULL");
                context.insert("initiative_id", "NULL");
                context.insert("parent_title", "Sample Parent Initiative");
            }
            "adr" => {
                context.insert("number", &1);
                context.insert("decision_maker", "");
                context.insert("decision_date", "");
            }
            _ => {}
        }

        context
    }

    /// Check if custom templates exist for a document type.
    pub fn has_custom_template(&self, doc_type: &str, template_type: TemplateType) -> bool {
        // Check project-level
        if let Some(ref project_path) = self.project_path {
            let project_template = project_path
                .join("templates")
                .join(doc_type)
                .join(template_type.filename());
            if project_template.exists() {
                return true;
            }
        }

        // Check global-level
        let global_template = self
            .global_path
            .join("templates")
            .join(doc_type)
            .join(template_type.filename());
        global_template.exists()
    }

    /// Get the source of a template (for debugging/info).
    pub fn template_source(&self, doc_type: &str, template_type: TemplateType) -> TemplateSource {
        // Check project-level
        if let Some(ref project_path) = self.project_path {
            let project_template = project_path
                .join("templates")
                .join(doc_type)
                .join(template_type.filename());
            if project_template.exists() {
                return TemplateSource::Project(project_template);
            }
        }

        // Check global-level
        let global_template = self
            .global_path
            .join("templates")
            .join(doc_type)
            .join(template_type.filename());
        if global_template.exists() {
            return TemplateSource::Global(global_template);
        }

        TemplateSource::Embedded
    }
}

/// Indicates where a template was loaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    /// Template from project's `.metis/templates/` directory
    Project(PathBuf),
    /// Template from global `~/.config/metis/templates/` directory
    Global(PathBuf),
    /// Embedded default template
    Embedded,
}

impl std::fmt::Display for TemplateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateSource::Project(path) => write!(f, "project: {}", path.display()),
            TemplateSource::Global(path) => write!(f, "global: {}", path.display()),
            TemplateSource::Embedded => write!(f, "embedded default"),
        }
    }
}

/// Helper to get the type letter for short codes
fn doc_type_letter(doc_type: &str) -> char {
    match doc_type {
        "vision" => 'V',
        "strategy" => 'S',
        "initiative" => 'I',
        "task" => 'T',
        "adr" => 'A',
        _ => 'X',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_embedded_templates() {
        let loader = TemplateLoader::new(None);

        // All document types should have embedded templates
        for doc_type in &["vision", "strategy", "initiative", "task", "adr"] {
            let content = loader.load_content_template(doc_type);
            assert!(content.is_ok(), "Failed to load content for {}", doc_type);
            assert!(!content.unwrap().is_empty());

            let exit_criteria = loader.load_exit_criteria_template(doc_type);
            assert!(
                exit_criteria.is_ok(),
                "Failed to load exit criteria for {}",
                doc_type
            );
        }
    }

    #[test]
    fn test_unknown_document_type() {
        let loader = TemplateLoader::new(None);
        let result = loader.load_content_template("unknown");
        assert!(matches!(result, Err(TemplateError::UnknownDocumentType(_))));
    }

    #[test]
    fn test_project_template_override() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create a custom template
        let template_dir = project_path.join("templates").join("task");
        std::fs::create_dir_all(&template_dir).unwrap();

        let custom_template = "# {{ title }}\n\nCustom task template!";
        std::fs::write(template_dir.join("content.md"), custom_template).unwrap();

        let loader = TemplateLoader::for_workspace(&project_path);
        let content = loader.load_content_template("task").unwrap();

        assert!(content.contains("Custom task template!"));
        assert_eq!(
            loader.template_source("task", TemplateType::Content),
            TemplateSource::Project(template_dir.join("content.md"))
        );
    }

    #[test]
    fn test_template_validation_error() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create an invalid template (unclosed Tera tag)
        let template_dir = project_path.join("templates").join("task");
        std::fs::create_dir_all(&template_dir).unwrap();

        let invalid_template = "# {{ title }\n\nBroken template";
        std::fs::write(template_dir.join("content.md"), invalid_template).unwrap();

        let loader = TemplateLoader::for_workspace(&project_path);
        let result = loader.load_content_template("task");

        assert!(matches!(result, Err(TemplateError::ParseError(_))));
    }

    #[test]
    fn test_template_validation_missing_variable() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create a template with an undefined variable
        let template_dir = project_path.join("templates").join("task");
        std::fs::create_dir_all(&template_dir).unwrap();

        let template_with_missing_var = "# {{ title }}\n\nValue: {{ nonexistent_variable }}";
        std::fs::write(template_dir.join("content.md"), template_with_missing_var).unwrap();

        let loader = TemplateLoader::for_workspace(&project_path);
        let result = loader.load_content_template("task");

        assert!(matches!(result, Err(TemplateError::ValidationError(_))));
    }

    #[test]
    fn test_sample_context_generation() {
        let loader = TemplateLoader::new(None);

        for doc_type in &["vision", "strategy", "initiative", "task", "adr"] {
            let context = loader.sample_context_for_type(doc_type);

            // All types should have common variables
            assert!(context.get("title").is_some());
            assert!(context.get("slug").is_some());
            assert!(context.get("short_code").is_some());
        }

        // Type-specific variables
        let initiative_ctx = loader.sample_context_for_type("initiative");
        assert!(initiative_ctx.get("estimated_complexity").is_some());

        let strategy_ctx = loader.sample_context_for_type("strategy");
        assert!(strategy_ctx.get("risk_level").is_some());
    }

    #[test]
    fn test_has_custom_template() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let loader = TemplateLoader::for_workspace(&project_path);

        // No custom templates initially
        assert!(!loader.has_custom_template("task", TemplateType::Content));

        // Create a custom template
        let template_dir = project_path.join("templates").join("task");
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(template_dir.join("content.md"), "# {{ title }}").unwrap();

        assert!(loader.has_custom_template("task", TemplateType::Content));
        assert!(!loader.has_custom_template("task", TemplateType::ExitCriteria));
    }
}
