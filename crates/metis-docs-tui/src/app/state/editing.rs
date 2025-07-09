use crate::models::EditState;

/// Editing state management for document editing operations
#[derive(Debug, Clone)]
pub struct EditingState {
    pub edit_state: Option<EditState>,
    pub current_field: usize,
    pub field_count: usize,
}

impl EditingState {
    pub fn new() -> Self {
        Self {
            edit_state: None,
            current_field: 0,
            field_count: 0,
        }
    }

    pub fn start_editing(&mut self, edit_state: EditState, field_count: usize) {
        self.edit_state = Some(edit_state);
        self.current_field = 0;
        self.field_count = field_count;
    }

    pub fn stop_editing(&mut self) {
        self.edit_state = None;
        self.current_field = 0;
        self.field_count = 0;
    }

    pub fn is_editing(&self) -> bool {
        self.edit_state.is_some()
    }

    pub fn next_field(&mut self) {
        if self.current_field < self.field_count.saturating_sub(1) {
            self.current_field += 1;
        }
    }

    pub fn previous_field(&mut self) {
        if self.current_field > 0 {
            self.current_field -= 1;
        }
    }

    pub fn get_current_field(&self) -> usize {
        self.current_field
    }

    pub fn get_edit_state(&self) -> Option<&EditState> {
        self.edit_state.as_ref()
    }

    pub fn get_edit_state_mut(&mut self) -> Option<&mut EditState> {
        self.edit_state.as_mut()
    }
}

impl Default for EditingState {
    fn default() -> Self {
        Self::new()
    }
}