use super::AppError;

/// Centralized error handler for the application
pub struct ErrorHandler {
    current_error: Option<AppError>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            current_error: None,
        }
    }

    pub fn handle_error(&mut self, error: AppError) {
        // Log the error (in a real implementation, you'd use a proper logger)
        eprintln!("Application Error: {}", error);

        // Store the error for UI display
        self.current_error = Some(error);
    }

    /// Handle common error patterns and provide user-friendly messages
    pub fn handle_with_context(&mut self, error: AppError, context: &str) {
        let contextual_error = match error {
            AppError::WorkspaceError(msg) => {
                AppError::WorkspaceError(format!("{}: {}", context, msg))
            }
            AppError::DocumentError(msg) => {
                AppError::DocumentError(format!("{}: {}", context, msg))
            }
            AppError::ValidationError(msg) => {
                AppError::ValidationError(format!("{}: {}", context, msg))
            }
            AppError::IoError(msg) => AppError::IoError(format!("{}: {}", context, msg)),
            AppError::DatabaseError(msg) => {
                AppError::DatabaseError(format!("{}: {}", context, msg))
            }
            AppError::UserInputError(msg) => {
                AppError::UserInputError(format!("{}: {}", context, msg))
            }
        };

        self.handle_error(contextual_error);
    }

    /// Convert various error types to user-friendly messages
    pub fn get_user_friendly_message(&self) -> Option<String> {
        self.current_error.as_ref().map(|error| match error {
            AppError::WorkspaceError(msg) => {
                if msg.contains("not in a Metis workspace") {
                    "Run 'metis init' to create a workspace".to_string()
                } else if msg.contains("database missing") {
                    "Run 'metis sync' to initialize the database".to_string()
                } else {
                    format!("Workspace issue: {}", msg)
                }
            }
            AppError::DocumentError(msg) => {
                if msg.contains("parent") {
                    "Invalid parent document selected".to_string()
                } else {
                    format!("Document issue: {}", msg)
                }
            }
            AppError::ValidationError(msg) => {
                format!("Validation failed: {}", msg)
            }
            AppError::UserInputError(msg) => {
                format!("Invalid input: {}", msg)
            }
            _ => error.to_string(),
        })
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}
