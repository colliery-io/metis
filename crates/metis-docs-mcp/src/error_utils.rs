use rust_mcp_sdk::schema::schema_utils::CallToolError;

/// Helper function to create CallToolError from string messages
pub fn tool_error(msg: &str) -> CallToolError {
    CallToolError::new(std::io::Error::other(msg))
}

/// Helper function to create CallToolError from formatted string
#[macro_export]
macro_rules! tool_error {
    ($($arg:tt)*) => {
        $crate::error_utils::tool_error(&format!($($arg)*))
    };
}
