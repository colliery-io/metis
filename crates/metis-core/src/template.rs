//! Template engine for document rendering

use crate::context::DocumentContext;
use crate::{DocumentType, MetisError, Result};
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use tera::{Context, Tera};

/// Template engine for rendering documents
static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/templates");

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Create a new template engine with all bundled templates
    pub fn new() -> Result<Self> {
        let mut tera = Tera::new("templates/**/*").unwrap_or_default();

        // Load templates from bundled directory
        let mut templates = HashMap::new();

        fn load_templates_recursive(
            dir: &Dir,
            path_prefix: &str,
            templates: &mut HashMap<String, String>,
        ) -> Result<()> {
            for entry in dir.entries() {
                match entry {
                    include_dir::DirEntry::Dir(subdir) => {
                        let new_prefix = if path_prefix.is_empty() {
                            subdir
                                .path()
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string()
                        } else {
                            format!(
                                "{}/{}",
                                path_prefix,
                                subdir.path().file_name().unwrap().to_string_lossy()
                            )
                        };
                        load_templates_recursive(subdir, &new_prefix, templates)?;
                    }
                    include_dir::DirEntry::File(file) => {
                        let filename = file.path().file_name().unwrap().to_string_lossy();
                        let template_name = if path_prefix.is_empty() {
                            filename.to_string()
                        } else {
                            format!("{}/{}", path_prefix, filename)
                        };

                        let content =
                            file.contents_utf8()
                                .ok_or_else(|| MetisError::ValidationFailed {
                                    message: format!(
                                        "Template {} is not valid UTF-8",
                                        template_name
                                    ),
                                })?;

                        templates.insert(template_name, content.to_string());
                    }
                }
            }
            Ok(())
        }

        load_templates_recursive(&TEMPLATES_DIR, "", &mut templates)?;

        // Add templates to Tera
        for (name, content) in templates {
            tera.add_raw_template(&name, &content)
                .map_err(|e| MetisError::ValidationFailed {
                    message: format!("Failed to parse template {}: {}", name, e),
                })?;
        }

        Ok(Self { tera })
    }

    /// Render a complete document (frontmatter + content + postmatter)
    pub fn render_document(
        &self,
        doc_type: &DocumentType,
        context: &DocumentContext,
    ) -> Result<String> {
        // Validate context for document type
        context.validate_for_type(doc_type)?;

        let type_name = match doc_type {
            DocumentType::Strategy => "strategy",
            DocumentType::Initiative => "initiative",
            DocumentType::Task => "task",
            DocumentType::Vision => "vision",
            DocumentType::Adr => "adr",
        };

        // Create Tera context
        let mut tera_context = Context::new();
        tera_context.insert("title", &context.title);
        tera_context.insert("slug", &context.slug);
        tera_context.insert(
            "created_at",
            &context.created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        );
        tera_context.insert(
            "updated_at",
            &context.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        );

        if let Some(ref parent) = context.parent_title {
            tera_context.insert("parent_title", parent);
            // Generate parent ID based on the parent title
            // For now, we'll infer the parent type from the document type hierarchy
            let parent_id = match doc_type {
                DocumentType::Strategy => "metis-vision".to_string(), // Strategies have vision as parent
                DocumentType::Initiative => {
                    format!("strategy-{}", DocumentContext::title_to_slug(parent))
                }
                DocumentType::Task => {
                    format!("initiative-{}", DocumentContext::title_to_slug(parent))
                }
                _ => DocumentContext::title_to_slug(parent), // Default case
            };
            tera_context.insert("parent_id", &parent_id);
        } else {
            tera_context.insert("parent_title", "");
            tera_context.insert("parent_id", "");
        }

        tera_context.insert("blocked_by", &context.blocked_by);
        tera_context.insert("stakeholders", &context.stakeholders);

        // Always provide these fields to templates even if empty
        tera_context.insert(
            "technical_lead",
            &context.technical_lead.as_deref().unwrap_or(""),
        );
        tera_context.insert(
            "decision_maker",
            &context.decision_maker.as_deref().unwrap_or(""),
        );
        if let Some(decision_date) = context.decision_date {
            tera_context.insert(
                "decision_date",
                &decision_date.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            );
        } else {
            tera_context.insert("decision_date", "");
        }
        if let Some(number) = context.number {
            tera_context.insert("number", &number);
        } else {
            tera_context.insert("number", &0u32);
        }
        if let Some(complexity) = context.complexity {
            tera_context.insert("complexity", &format!("{:?}", complexity));
        } else {
            tera_context.insert("complexity", "");
        }
        if let Some(risk_level) = context.risk_level {
            tera_context.insert("risk_level", &format!("{:?}", risk_level).to_lowercase());
        } else {
            tera_context.insert("risk_level", "medium");
        }

        // Render each part
        let frontmatter =
            self.render_template(&format!("{}/frontmatter.yaml", type_name), &tera_context)?;
        let content = self.render_template(&format!("{}/content.md", type_name), &tera_context)?;
        let postmatter =
            self.render_template(&format!("{}/postmatter.md", type_name), &tera_context)?;

        // Combine with proper YAML frontmatter formatting
        Ok(format!(
            "---\n{}\n---\n\n{}\n\n{}",
            frontmatter, content, postmatter
        ))
    }

    /// Render a specific template with context
    pub fn render_template(&self, template_name: &str, context: &Context) -> Result<String> {
        self.tera
            .render(template_name, context)
            .map_err(|e| MetisError::ValidationFailed {
                message: format!("Template rendering failed for {}: {}", template_name, e),
            })
    }

    /// Generate destination path for a document based on type and context
    pub fn generate_destination_path(
        &self,
        doc_type: &DocumentType,
        context: &DocumentContext,
    ) -> String {
        match doc_type {
            DocumentType::Vision => "vision.md".to_string(),
            DocumentType::Strategy => {
                format!("strategies/{}/strategy.md", context.slug)
            }
            DocumentType::Initiative => {
                if let Some(ref parent) = context.parent_title {
                    let parent_slug = DocumentContext::title_to_slug(parent);
                    format!(
                        "strategies/{}/initiatives/{}/initiative.md",
                        parent_slug, context.slug
                    )
                } else {
                    format!("initiatives/{}/initiative.md", context.slug)
                }
            }
            DocumentType::Task => {
                if let Some(ref parent) = context.parent_title {
                    // For tasks, we need to traverse up to find the strategy
                    // This is a simplified version - in practice we'd query the database
                    let parent_slug = DocumentContext::title_to_slug(parent);
                    format!(
                        "strategies/*/initiatives/{}/tasks/{}.md",
                        parent_slug, context.slug
                    )
                } else {
                    format!("tasks/{}.md", context.slug)
                }
            }
            DocumentType::Adr => {
                if let Some(number) = context.number {
                    format!("decisions/adr-{:03}-{}.md", number, context.slug)
                } else {
                    format!("decisions/{}.md", context.slug)
                }
            }
        }
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.tera
            .get_template_names()
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Complexity, RiskLevel};

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        let templates = engine.list_templates();
        eprintln!("Available templates: {:?}", templates);
        assert!(!templates.is_empty());

        // Check that we have templates for all document types
        assert!(templates.iter().any(|t| t.contains("strategy")));
        assert!(templates.iter().any(|t| t.contains("initiative")));
        assert!(templates.iter().any(|t| t.contains("task")));
        assert!(templates.iter().any(|t| t.contains("vision")));
        assert!(templates.iter().any(|t| t.contains("adr")));
    }

    #[test]
    fn test_strategy_document_rendering() {
        let engine = TemplateEngine::new().unwrap();
        let context = DocumentContext::new("Test Strategy".to_string())
            .with_risk_level(RiskLevel::High)
            .with_stakeholders(vec!["Engineering".to_string(), "Product".to_string()]);

        let result = engine.render_document(&DocumentType::Strategy, &context);
        if let Err(ref e) = result {
            eprintln!("Render error: {:?}", e);
        }
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("---\n")); // Has frontmatter
        assert!(rendered.contains("# Test Strategy Strategy")); // Has content
        assert!(rendered.contains("test-strategy")); // Has slug
        assert!(rendered.contains("high")); // Has risk level
    }

    #[test]
    fn test_initiative_document_rendering() {
        let engine = TemplateEngine::new().unwrap();
        let context = DocumentContext::new("Test Initiative".to_string())
            .with_parent("Parent Strategy".to_string())
            .with_complexity(Complexity::L)
            .with_technical_lead("Alice Smith".to_string());

        let result = engine.render_document(&DocumentType::Initiative, &context);
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("---\n")); // Has frontmatter
        assert!(rendered.contains("# Test Initiative")); // Has content
        assert!(rendered.contains("strategy-parent-strategy")); // Has parent ID
        assert!(rendered.contains("L")); // Has complexity
        assert!(rendered.contains("Alice Smith")); // Has technical lead
    }

    #[test]
    fn test_adr_document_rendering() {
        let engine = TemplateEngine::new().unwrap();
        let context = DocumentContext::new("Use Database for Storage".to_string())
            .with_decision_maker("Architecture Team".to_string())
            .with_number(5);

        let result = engine.render_document(&DocumentType::Adr, &context);
        if let Err(ref e) = result {
            eprintln!("ADR render error: {:?}", e);
        }
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(rendered.contains("---\n")); // Has frontmatter
        assert!(rendered.contains("Use Database for Storage")); // Has title
        assert!(rendered.contains("Architecture Team")); // Has decision maker
        assert!(rendered.contains("5")); // Has ADR number
    }

    #[test]
    fn test_validation_failure_on_missing_required_fields() {
        let engine = TemplateEngine::new().unwrap();

        // Strategy without risk_level should fail
        let strategy_context = DocumentContext::new("Test Strategy".to_string());
        let result = engine.render_document(&DocumentType::Strategy, &strategy_context);
        assert!(result.is_err());

        // Initiative without complexity should fail
        let initiative_context = DocumentContext::new("Test Initiative".to_string());
        let result = engine.render_document(&DocumentType::Initiative, &initiative_context);
        assert!(result.is_err());

        // ADR without decision_maker should fail
        let adr_context = DocumentContext::new("Test ADR".to_string());
        let result = engine.render_document(&DocumentType::Adr, &adr_context);
        assert!(result.is_err());
    }

    #[test]
    fn test_destination_path_generation() {
        let engine = TemplateEngine::new().unwrap();

        // Vision
        let vision_context = DocumentContext::new("Product Vision".to_string());
        assert_eq!(
            engine.generate_destination_path(&DocumentType::Vision, &vision_context),
            "vision.md"
        );

        // Strategy
        let strategy_context = DocumentContext::new("Core Platform Strategy".to_string());
        assert_eq!(
            engine.generate_destination_path(&DocumentType::Strategy, &strategy_context),
            "strategies/core-platform-strategy/strategy.md"
        );

        // Initiative with parent
        let initiative_context = DocumentContext::new("API Design".to_string())
            .with_parent("Core Platform Strategy".to_string());
        assert_eq!(
            engine.generate_destination_path(&DocumentType::Initiative, &initiative_context),
            "strategies/core-platform-strategy/initiatives/api-design/initiative.md"
        );

        // ADR with number
        let adr_context = DocumentContext::new("Use GraphQL".to_string()).with_number(42);
        assert_eq!(
            engine.generate_destination_path(&DocumentType::Adr, &adr_context),
            "decisions/adr-042-use-graphql.md"
        );
    }

    #[test]
    fn test_vision_and_task_render_without_extra_fields() {
        let engine = TemplateEngine::new().unwrap();

        // Vision should render without any extra fields
        let vision_context = DocumentContext::new("Product Vision".to_string());
        let result = engine.render_document(&DocumentType::Vision, &vision_context);
        assert!(result.is_ok());

        // Task should render without any extra fields
        let task_context = DocumentContext::new("Implement Feature".to_string());
        let result = engine.render_document(&DocumentType::Task, &task_context);
        assert!(result.is_ok());
    }
}
