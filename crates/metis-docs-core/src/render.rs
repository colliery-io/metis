//! Main render function for creating documents

use crate::context::DocumentContext;
use crate::template::TemplateEngine;
use crate::{DocumentType, MetisError, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Main render function: create document from template and write to filesystem
pub async fn render(
    document_type: DocumentType,
    context: DocumentContext,
    docs_root: &Path,
) -> Result<PathBuf> {
    // Create template engine
    let engine = TemplateEngine::new()?;

    // Validate context for document type
    context.validate_for_type(&document_type)?;

    // Render document content
    let content = engine.render_document(&document_type, &context)?;

    // Generate destination path
    let relative_path = engine.generate_destination_path(&document_type, &context);
    let full_path = docs_root.join(&relative_path);

    // Create parent directories if they don't exist
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).await.map_err(MetisError::Io)?;
    }

    // Write file to filesystem
    fs::write(&full_path, content)
        .await
        .map_err(MetisError::Io)?;

    Ok(full_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Complexity, RiskLevel};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_render_strategy_to_filesystem() {
        let temp_dir = TempDir::new().unwrap();
        let docs_root = temp_dir.path();

        let context = DocumentContext::new("Test Strategy".to_string())
            .with_risk_level(RiskLevel::High)
            .with_strategy_id("test-strategy".to_string())
            .with_stakeholders(vec!["Engineering".to_string(), "Product".to_string()]);

        let result = render(DocumentType::Strategy, context, docs_root).await;
        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.exists());
        assert!(file_path
            .to_string_lossy()
            .contains("strategies/test-strategy/strategy.md"));

        // Verify file contents
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---\n")); // Has frontmatter
        assert!(content.contains("# Test Strategy Strategy")); // Has content
        assert!(content.contains("high")); // Has risk level
    }

    #[tokio::test]
    async fn test_render_initiative_with_parent() {
        let temp_dir = TempDir::new().unwrap();
        let docs_root = temp_dir.path();

        let context = DocumentContext::new("API Design".to_string())
            .with_parent("Core Platform Strategy".to_string())
            .with_strategy_id("core-platform-strategy".to_string())
            .with_complexity(Complexity::L)
            .with_technical_lead("Alice Smith".to_string());

        let result = render(DocumentType::Initiative, context, docs_root).await;
        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.exists());
        assert!(file_path
            .to_string_lossy()
            .contains("strategies/core-platform-strategy/initiatives/api-design/initiative.md"));

        // Verify file contents
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("strategy-core-platform-strategy")); // Has parent ID
        assert!(content.contains("L")); // Has complexity
        assert!(content.contains("Alice Smith")); // Has technical lead
    }

    #[tokio::test]
    async fn test_render_adr_with_number() {
        let temp_dir = TempDir::new().unwrap();
        let docs_root = temp_dir.path();

        let context = DocumentContext::new("Use GraphQL".to_string())
            .with_decision_maker("Architecture Team".to_string())
            .with_strategy_id("test-strategy".to_string())
            .with_number(42);

        let result = render(DocumentType::Adr, context, docs_root).await;
        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.exists());
        assert!(file_path
            .to_string_lossy()
            .contains("decisions/adr-042-use-graphql.md"));

        // Verify file contents
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("ADR-42: Use GraphQL")); // Has title
        assert!(content.contains("Architecture Team")); // Has decision maker
    }

    #[tokio::test]
    async fn test_render_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let docs_root = temp_dir.path();

        let context = DocumentContext::new("Deep Nested Strategy".to_string())
            .with_risk_level(RiskLevel::Medium)
            .with_strategy_id("deep-nested-strategy".to_string());

        // Ensure the directory doesn't exist initially
        let expected_dir = docs_root.join("strategies/deep-nested-strategy");
        assert!(!expected_dir.exists());

        let result = render(DocumentType::Strategy, context, docs_root).await;
        assert!(result.is_ok());

        // Verify directory was created
        assert!(expected_dir.exists());
        assert!(expected_dir.is_dir());

        // Verify file was created
        let file_path = result.unwrap();
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_render_validation_failure() {
        let temp_dir = TempDir::new().unwrap();
        let docs_root = temp_dir.path();

        // Strategy without required risk_level should fail
        let context = DocumentContext::new("Invalid Strategy".to_string());

        let result = render(DocumentType::Strategy, context, docs_root).await;
        assert!(result.is_err());

        // Verify no file was created
        let expected_path = docs_root.join("strategies/invalid-strategy/strategy.md");
        assert!(!expected_path.exists());
    }

    #[tokio::test]
    async fn test_render_vision_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let docs_root = temp_dir.path();

        let context = DocumentContext::new("Product Vision".to_string())
            .with_stakeholders(vec!["Product".to_string(), "Engineering".to_string()]);

        let result = render(DocumentType::Vision, context, docs_root).await;
        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.exists());
        assert_eq!(file_path, docs_root.join("vision.md"));

        // Verify file contents
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("Product Vision"));
    }
}
