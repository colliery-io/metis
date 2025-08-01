use std::time::{Duration, Instant};

/// Type of message to display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Error,
    Success,
    Warning,
    Info,
}

impl MessageType {
    /// Get the display duration for this message type
    pub fn duration(&self) -> Duration {
        match self {
            MessageType::Error => Duration::from_secs(10),
            MessageType::Success => Duration::from_secs(5),
            MessageType::Warning => Duration::from_secs(7),
            MessageType::Info => Duration::from_secs(4),
        }
    }

    /// Get the display prefix for this message type
    pub fn prefix(&self) -> &'static str {
        match self {
            MessageType::Error => "ERROR",
            MessageType::Success => "SUCCESS",
            MessageType::Warning => "WARNING",
            MessageType::Info => "INFO",
        }
    }
}

/// A message to display to the user
#[derive(Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub message_type: MessageType,
    pub content: String,
    pub created_at: Instant,
    pub auto_clear: bool,
}

impl Message {
    /// Create a new message
    pub fn new(id: u64, message_type: MessageType, content: String) -> Self {
        Self {
            id,
            message_type,
            content,
            created_at: Instant::now(),
            auto_clear: true,
        }
    }

    /// Create a message that won't auto-clear
    pub fn persistent(id: u64, message_type: MessageType, content: String) -> Self {
        Self {
            id,
            message_type,
            content,
            created_at: Instant::now(),
            auto_clear: false,
        }
    }

    /// Check if this message should be cleared based on its age
    pub fn should_clear(&self) -> bool {
        if !self.auto_clear {
            return false;
        }
        
        let age = self.created_at.elapsed();
        age >= self.message_type.duration()
    }

    /// Get display text for the message
    pub fn display_text(&self) -> String {
        format!("{}: {}", self.message_type.prefix(), self.content)
    }
}

/// State container for messages
#[derive(Debug)]
pub struct MessageState {
    pub current_message: Option<Message>,
    next_id: u64,
}

impl Default for MessageState {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageState {
    /// Create a new message state
    pub fn new() -> Self {
        Self {
            current_message: None,
            next_id: 1,
        }
    }

    /// Set the current message (replaces any existing message)
    fn set_message(&mut self, message_type: MessageType, content: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let message = Message::new(id, message_type, content);
        self.current_message = Some(message);

        id
    }

    /// Add an error message
    pub fn add_error(&mut self, content: String) -> u64 {
        self.set_message(MessageType::Error, content)
    }

    /// Add a success message
    pub fn add_success(&mut self, content: String) -> u64 {
        self.set_message(MessageType::Success, content)
    }

    /// Add a warning message
    pub fn add_warning(&mut self, content: String) -> u64 {
        self.set_message(MessageType::Warning, content)
    }

    /// Add an info message
    pub fn add_info(&mut self, content: String) -> u64 {
        self.set_message(MessageType::Info, content)
    }

    /// Clear the current message
    pub fn clear_message(&mut self) {
        self.current_message = None;
    }

    /// Clear expired messages
    pub fn clear_expired_messages(&mut self) {
        if let Some(ref msg) = self.current_message {
            if msg.should_clear() {
                self.current_message = None;
            }
        }
    }

    /// Get the current message if it's not expired
    pub fn get_current_message(&self) -> Option<&Message> {
        self.current_message.as_ref().filter(|msg| !msg.should_clear())
    }

    /// Check if there is an active message
    pub fn has_messages(&self) -> bool {
        self.get_current_message().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(1, MessageType::Error, "Test error".to_string());
        assert_eq!(msg.id, 1);
        assert_eq!(msg.message_type, MessageType::Error);
        assert_eq!(msg.content, "Test error");
        assert!(msg.auto_clear);
    }

    #[test]
    fn test_persistent_message() {
        let msg = Message::persistent(1, MessageType::Info, "Persistent info".to_string());
        assert!(!msg.auto_clear);
        assert!(!msg.should_clear());
    }

    #[test]
    fn test_message_display_text() {
        let msg = Message::new(1, MessageType::Success, "Operation completed".to_string());
        assert_eq!(msg.display_text(), "SUCCESS: Operation completed");
    }

    #[test]
    fn test_message_state_operations() {
        let mut state = MessageState::new();
        
        // Initially no message
        assert!(!state.has_messages());
        
        // Add error message
        state.add_error("Error 1".to_string());
        assert!(state.has_messages());
        
        // Add success message (replaces error)
        state.add_success("Success 1".to_string());
        assert!(state.has_messages());
        if let Some(msg) = state.get_current_message() {
            assert_eq!(msg.message_type, MessageType::Success);
            assert_eq!(msg.content, "Success 1");
        }
        
        // Clear message
        state.clear_message();
        assert!(!state.has_messages());
    }
}