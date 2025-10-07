//! Error conversion traits and utilities for consistent error handling across crates

use crate::error::MetisError;

/// Trait for converting errors with additional context
pub trait ErrorContext<T> {
    /// Add context to an error result
    fn with_context<F>(self, f: F) -> Result<T, MetisError>
    where
        F: FnOnce() -> String;

    /// Add static context to an error result
    fn with_static_context(self, context: &'static str) -> Result<T, MetisError>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<MetisError>,
{
    fn with_context<F>(self, f: F) -> Result<T, MetisError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            MetisError::ValidationFailed {
                message: format!("{}: {}", f(), base_error),
            }
        })
    }

    fn with_static_context(self, context: &'static str) -> Result<T, MetisError> {
        self.map_err(|e| {
            let base_error = e.into();
            MetisError::ValidationFailed {
                message: format!("{}: {}", context, base_error),
            }
        })
    }
}

/// Trait for creating user-friendly error messages from MetisError
pub trait UserFriendlyError {
    /// Convert to a user-friendly error message
    fn to_user_message(&self) -> String;

    /// Get error category for UI display
    fn error_category(&self) -> ErrorCategory;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    Workspace,
    Document,
    Database,
    FileSystem,
    Validation,
    Network,
    Configuration,
}

impl UserFriendlyError for MetisError {
    fn to_user_message(&self) -> String {
        match self {
            MetisError::DocumentNotFound { id } => {
                format!(
                    "Document '{}' could not be found. It may have been moved or deleted.",
                    id
                )
            }
            MetisError::InvalidDocumentType { document_type } => {
                format!("'{}' is not a valid document type. Valid types are: vision, strategy, initiative, task, adr.", document_type)
            }
            MetisError::InvalidPhaseTransition { from, to, doc_type } => {
                format!("Cannot transition {} from '{}' to '{}'. Please check the valid phase transitions for this document type.", doc_type, from, to)
            }
            MetisError::MissingRequiredField { field } => {
                format!(
                    "Required field '{}' is missing. Please provide this information.",
                    field
                )
            }
            MetisError::TemplateNotFound { template } => {
                format!(
                    "Template '{}' could not be found. Please check your template configuration.",
                    template
                )
            }
            MetisError::ValidationFailed { message } => {
                format!("Validation failed: {}", message)
            }
            MetisError::ExitCriteriaNotMet {
                missing_count,
                total_count,
            } => {
                format!("{} of {} exit criteria are incomplete. Please complete all criteria before proceeding.", missing_count, total_count)
            }
            MetisError::Database(e) => {
                format!("Database error: {}. Please try again or contact support if the issue persists.", e)
            }
            MetisError::Connection(e) => {
                format!(
                    "Database connection error: {}. Please check your database configuration.",
                    e
                )
            }
            MetisError::Io(e) => {
                format!(
                    "File system error: {}. Please check file permissions and try again.",
                    e
                )
            }
            MetisError::Json(e) => {
                format!(
                    "JSON parsing error: {}. Please check the document format.",
                    e
                )
            }
            MetisError::Yaml(e) => {
                format!(
                    "YAML parsing error: {}. Please check the document format.",
                    e
                )
            }
            _ => {
                format!("An error occurred: {}. Please try again.", self)
            }
        }
    }

    fn error_category(&self) -> ErrorCategory {
        match self {
            MetisError::DocumentNotFound { .. }
            | MetisError::InvalidDocumentType { .. }
            | MetisError::TemplateNotFound { .. } => ErrorCategory::Document,

            MetisError::Database(_) | MetisError::Connection(_) => ErrorCategory::Database,

            MetisError::Io(_) => ErrorCategory::FileSystem,

            MetisError::InvalidPhaseTransition { .. }
            | MetisError::MissingRequiredField { .. }
            | MetisError::ValidationFailed { .. }
            | MetisError::ExitCriteriaNotMet { .. } => ErrorCategory::Validation,

            MetisError::Json(_) | MetisError::Yaml(_) => ErrorCategory::Document,

            _ => ErrorCategory::Configuration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let with_context = result.with_static_context("Reading configuration file");
        assert!(with_context.is_err());
        assert!(with_context
            .unwrap_err()
            .to_string()
            .contains("Reading configuration file"));
    }

    #[test]
    fn test_user_friendly_error_document_not_found() {
        let error = MetisError::DocumentNotFound {
            id: "test-doc".to_string(),
        };
        let message = error.to_user_message();
        assert!(message.contains("Document 'test-doc' could not be found"));
        assert_eq!(error.error_category(), ErrorCategory::Document);
    }

    #[test]
    fn test_user_friendly_error_validation() {
        let error = MetisError::ValidationFailed {
            message: "test validation".to_string(),
        };
        let message = error.to_user_message();
        assert!(message.contains("Validation failed: test validation"));
        assert_eq!(error.error_category(), ErrorCategory::Validation);
    }
}
