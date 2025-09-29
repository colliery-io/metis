use metis_core::{domain::documents::types::Phase, Adr, Document, Initiative, Strategy, Task};

/// Extract numerical part from ADR ID (e.g., "001-some-title" -> 1)
pub fn extract_adr_number(id: &str) -> u32 {
    if let Some(dash_pos) = id.find('-') {
        if let Ok(num) = id[..dash_pos].parse::<u32>() {
            return num;
        }
    }
    // Fallback: if parsing fails, return 0 to sort to beginning
    0
}

/// Get column index for strategy board based on phase
/// Columns: shaping(0), design(1), ready(2), active(3), completed(4)
pub fn get_strategy_column_index(strategy: &Strategy) -> usize {
    match strategy.phase() {
        Ok(Phase::Shaping) => 0,
        Ok(Phase::Design) => 1,
        Ok(Phase::Ready) => 2,
        Ok(Phase::Active) => 3,
        Ok(Phase::Completed) => 4,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for initiative board based on phase
/// Columns: discovery(0), design(1), ready(2), decompose(3), active(4), completed(5)
pub fn get_initiative_column_index(initiative: &Initiative) -> usize {
    match initiative.phase() {
        Ok(Phase::Discovery) => 0,
        Ok(Phase::Design) => 1,
        Ok(Phase::Ready) => 2,
        Ok(Phase::Decompose) => 3,
        Ok(Phase::Active) => 4,
        Ok(Phase::Completed) => 5,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for task board based on phase
/// Columns: todo(0), active(1), blocked(2), completed(3)
pub fn get_task_column_index(task: &Task) -> usize {
    match task.phase() {
        Ok(Phase::Todo) => 0,
        Ok(Phase::Active) => 1,
        Ok(Phase::Blocked) => 2,
        Ok(Phase::Completed) => 3,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for ADR board based on phase
/// Columns: draft(0), discussion(1), decided(2), superseded(3)
pub fn get_adr_column_index(adr: &Adr) -> usize {
    match adr.phase() {
        Ok(Phase::Draft) => 0,
        Ok(Phase::Discussion) => 1,
        Ok(Phase::Decided) => 2,
        Ok(Phase::Superseded) => 3,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for backlog board based on task type
/// Columns: backlog(0), bugs(1), features(2), tech-debt(3)
pub fn get_backlog_column_index(task: &Task) -> usize {
    use metis_core::domain::documents::types::Tag;
    
    // Check task tags to determine type
    for tag in &task.core().tags {
        if let Tag::Label(label) = tag {
            match label.as_str() {
                "bug" => return 1, // bugs column
                "feature" => return 2, // features column
                "tech-debt" => return 3, // tech-debt column
                _ => {}
            }
        }
    }
    
    // Default to backlog column (general items)
    0
}