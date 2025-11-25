//! Standardized output formatting for MCP tool responses.
//!
//! Provides a builder-style API for constructing consistent, readable markdown output
//! that renders well in terminal contexts.

use std::fmt::Write;

/// Status icons for visual indicators
pub struct Icons;

impl Icons {
    pub const SUCCESS: &'static str = "\u{2713}"; // ✓
    pub const ERROR: &'static str = "\u{2717}";   // ✗
    pub const WARNING: &'static str = "\u{26A0}"; // ⚠
    pub const INFO: &'static str = "\u{2139}";    // ℹ
    pub const PENDING: &'static str = "\u{25CB}"; // ○
    pub const ACTIVE: &'static str = "\u{25CF}";  // ●
}

/// Builder for constructing formatted tool output
#[derive(Default)]
pub struct ToolOutput {
    sections: Vec<String>,
}

impl ToolOutput {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an H2 header (primary section header)
    pub fn header(mut self, text: &str) -> Self {
        self.sections.push(format!("## {}", text));
        self
    }

    /// Add an H3 header (subsection header)
    pub fn subheader(mut self, text: &str) -> Self {
        self.sections.push(format!("### {}", text));
        self
    }

    /// Add plain text
    pub fn text(mut self, text: &str) -> Self {
        self.sections.push(text.to_string());
        self
    }

    /// Add an empty line for spacing
    pub fn blank(mut self) -> Self {
        self.sections.push(String::new());
        self
    }

    /// Add a horizontal rule
    pub fn rule(mut self) -> Self {
        self.sections.push("---".to_string());
        self
    }

    /// Add a success message with checkmark
    pub fn success(mut self, msg: &str) -> Self {
        self.sections.push(format!("{} {}", Icons::SUCCESS, msg));
        self
    }

    /// Add an error message with X
    pub fn error(mut self, msg: &str) -> Self {
        self.sections.push(format!("{} {}", Icons::ERROR, msg));
        self
    }

    /// Add a warning message
    pub fn warning(mut self, msg: &str) -> Self {
        self.sections.push(format!("{} {}", Icons::WARNING, msg));
        self
    }

    /// Add an info message
    pub fn info(mut self, msg: &str) -> Self {
        self.sections.push(format!("{} {}", Icons::INFO, msg));
        self
    }

    /// Add a key-value pair
    pub fn field(mut self, key: &str, value: &str) -> Self {
        self.sections.push(format!("**{}**: {}", key, value));
        self
    }

    /// Add inline code
    pub fn code_inline(mut self, code: &str) -> Self {
        self.sections.push(format!("`{}`", code));
        self
    }

    /// Add a fenced code block
    pub fn code_block(mut self, code: &str, lang: Option<&str>) -> Self {
        let fence = format!("```{}", lang.unwrap_or(""));
        self.sections.push(format!("{}\n{}\n```", fence, code));
        self
    }

    /// Add a diff block showing before/after
    pub fn diff(mut self, old: &str, new: &str) -> Self {
        let mut diff_content = String::new();
        for line in old.lines() {
            writeln!(diff_content, "- {}", line).unwrap();
        }
        for line in new.lines() {
            writeln!(diff_content, "+ {}", line).unwrap();
        }
        // Remove trailing newline
        if diff_content.ends_with('\n') {
            diff_content.pop();
        }
        self.sections.push(format!("```diff\n{}\n```", diff_content));
        self
    }

    /// Add a simple key-value table
    pub fn table(mut self, headers: &[&str], rows: Vec<Vec<String>>) -> Self {
        if headers.is_empty() || rows.is_empty() {
            return self;
        }

        let mut table = String::new();

        // Header row
        writeln!(table, "| {} |", headers.join(" | ")).unwrap();

        // Separator row
        let separators: Vec<&str> = headers.iter().map(|_| "---").collect();
        writeln!(table, "|{}|", separators.join("|")).unwrap();

        // Data rows
        for row in rows {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    if i < headers.len() {
                        cell.clone()
                    } else {
                        cell.clone()
                    }
                })
                .collect();
            writeln!(table, "| {} |", cells.join(" | ")).unwrap();
        }

        // Remove trailing newline
        if table.ends_with('\n') {
            table.pop();
        }

        self.sections.push(table);
        self
    }

    /// Add a list of items with status indicators
    pub fn status_list(mut self, items: Vec<(&str, bool)>) -> Self {
        let mut list = String::new();
        for (item, completed) in items {
            let icon = if completed { Icons::ACTIVE } else { Icons::PENDING };
            writeln!(list, "{} {}", icon, item).unwrap();
        }
        if list.ends_with('\n') {
            list.pop();
        }
        self.sections.push(list);
        self
    }

    /// Add a bulleted list
    pub fn bullet_list(mut self, items: &[&str]) -> Self {
        let list: Vec<String> = items.iter().map(|item| format!("- {}", item)).collect();
        self.sections.push(list.join("\n"));
        self
    }

    /// Add indented content (for nested items)
    pub fn indented(mut self, items: Vec<(bool, &str)>) -> Self {
        let mut content = String::new();
        for (success, item) in items {
            let icon = if success { Icons::SUCCESS } else { Icons::ERROR };
            writeln!(content, "  {} {}", icon, item).unwrap();
        }
        if content.ends_with('\n') {
            content.pop();
        }
        self.sections.push(content);
        self
    }

    /// Add a phase progression indicator
    pub fn phase_progress(mut self, phases: &[&str], current_index: usize) -> Self {
        let indicators: Vec<String> = phases
            .iter()
            .enumerate()
            .map(|(i, phase)| {
                let icon = if i <= current_index {
                    Icons::ACTIVE
                } else {
                    Icons::PENDING
                };
                format!("{} {}", icon, phase)
            })
            .collect();
        self.sections.push(indicators.join(" -> "));
        self
    }

    /// Add a hint/tip message
    pub fn hint(mut self, msg: &str) -> Self {
        self.sections.push(format!("**Hint**: {}", msg));
        self
    }

    /// Build the final output string
    pub fn build(self) -> String {
        self.sections.join("\n\n")
    }
}

/// Helper for formatting error responses consistently
pub fn format_error(title: &str, message: &str, hint: Option<&str>) -> String {
    let mut output = ToolOutput::new()
        .header("Error")
        .error(title)
        .blank()
        .text(message);

    if let Some(h) = hint {
        output = output.blank().hint(h);
    }

    output.build()
}

/// Helper for formatting not-found errors
pub fn format_not_found(resource_type: &str, identifier: &str, hint: Option<&str>) -> String {
    format_error(
        &format!("{} not found: {}", resource_type, identifier),
        &format!(
            "No {} with identifier \"{}\" exists in this project.",
            resource_type.to_lowercase(),
            identifier
        ),
        hint,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_output() {
        let output = ToolOutput::new()
            .header("Document Created")
            .success("METIS-T-0001 created successfully")
            .build();

        assert!(output.contains("## Document Created"));
        assert!(output.contains(Icons::SUCCESS));
        assert!(output.contains("METIS-T-0001"));
    }

    #[test]
    fn test_table_output() {
        let output = ToolOutput::new()
            .header("Test")
            .table(
                &["Code", "Title"],
                vec![
                    vec!["METIS-T-0001".to_string(), "Test Task".to_string()],
                ],
            )
            .build();

        assert!(output.contains("| Code | Title |"));
        assert!(output.contains("METIS-T-0001"));
    }

    #[test]
    fn test_diff_output() {
        let output = ToolOutput::new()
            .header("Change")
            .diff("old text", "new text")
            .build();

        assert!(output.contains("```diff"));
        assert!(output.contains("- old text"));
        assert!(output.contains("+ new text"));
    }

    #[test]
    fn test_phase_progress() {
        let output = ToolOutput::new()
            .phase_progress(&["todo", "doing", "done"], 1)
            .build();

        assert!(output.contains(Icons::ACTIVE));
        assert!(output.contains(Icons::PENDING));
        assert!(output.contains("todo"));
        assert!(output.contains("doing"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_error_formatting() {
        let output = format_error(
            "Something went wrong",
            "Detailed explanation here.",
            Some("Try doing X instead."),
        );

        assert!(output.contains("## Error"));
        assert!(output.contains(Icons::ERROR));
        assert!(output.contains("**Hint**"));
    }
}
