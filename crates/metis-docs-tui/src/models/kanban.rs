use metis_core::{
    domain::documents::types::DocumentType,
    Strategy, Initiative, Task, Adr, Document,
};

#[derive(Debug)]
pub enum DocumentObject {
    Strategy(Strategy),
    Initiative(Initiative),
    Task(Task),
    Adr(Adr),
}

impl DocumentObject {
    pub fn as_document(&self) -> &dyn Document {
        match self {
            DocumentObject::Strategy(doc) => doc,
            DocumentObject::Initiative(doc) => doc,
            DocumentObject::Task(doc) => doc,
            DocumentObject::Adr(doc) => doc,
        }
    }
    
    pub fn as_document_mut(&mut self) -> &mut dyn Document {
        match self {
            DocumentObject::Strategy(doc) => doc,
            DocumentObject::Initiative(doc) => doc,
            DocumentObject::Task(doc) => doc,
            DocumentObject::Adr(doc) => doc,
        }
    }
}

#[derive(Debug)]
pub struct KanbanItem {
    pub document: DocumentObject,
    pub prelude: String, // Cached display text
    pub risk_complexity: Option<String>, // Cached display text for risk/complexity
    pub file_path: String,
}

impl KanbanItem {
    pub fn id(&self) -> String {
        self.document.as_document().id().to_string()
    }
    
    pub fn title(&self) -> &str {
        self.document.as_document().title()
    }
    
    pub fn doc_type(&self) -> DocumentType {
        self.document.as_document().document_type()
    }
    
    pub fn phase(&self) -> String {
        self.document.as_document().phase()
            .map(|p| p.to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    }
    
    pub fn blocked_by(&self) -> Vec<String> {
        self.document.as_document().blocked_by()
            .iter()
            .map(|id| id.to_string())
            .collect()
    }
    
    pub fn parent_title(&self) -> Option<String> {
        self.document.as_document().parent_id()
            .map(|id| id.to_string())
    }
}

#[derive(Debug)]
pub struct KanbanBoard {
    pub title: String,
    pub columns: Vec<KanbanColumn>,
}

#[derive(Debug)]
pub struct KanbanColumn {
    pub title: String,
    pub items: Vec<KanbanItem>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BoardType {
    Strategy,
    Initiative,
    Task,
}

#[derive(Debug, PartialEq)]
pub enum AppState {
    Normal,
    CreatingDocument,
    CreatingChildDocument,
    EditingDocument,
    EditingStrategy,
    ConfirmingDelete,
}

#[derive(Debug, Clone)]
pub enum EditField {
    Title,
    Description,
    // Add more fields as needed
}

#[derive(Debug, Clone)]
pub struct EditState {
    pub current_field: EditField,
    pub title: String,
    pub description: String,
    pub original_item_id: String,
    pub original_item_title: String,
}

impl KanbanBoard {
    pub fn create_strategy_board() -> Self {
        Self {
            title: "Strategies".to_string(),
            columns: vec![
                KanbanColumn { title: "shaping".to_string(), items: vec![] },
                KanbanColumn { title: "design".to_string(), items: vec![] },
                KanbanColumn { title: "ready".to_string(), items: vec![] },
                KanbanColumn { title: "active".to_string(), items: vec![] },
                KanbanColumn { title: "completed".to_string(), items: vec![] },
            ],
        }
    }

    pub fn create_initiative_board() -> Self {
        Self {
            title: "Initiatives".to_string(),
            columns: vec![
                KanbanColumn { title: "discovery".to_string(), items: vec![] },
                KanbanColumn { title: "design".to_string(), items: vec![] },
                KanbanColumn { title: "ready".to_string(), items: vec![] },
                KanbanColumn { title: "decompose".to_string(), items: vec![] },
                KanbanColumn { title: "active".to_string(), items: vec![] },
                KanbanColumn { title: "completed".to_string(), items: vec![] },
            ],
        }
    }

    pub fn create_task_board() -> Self {
        Self {
            title: "Tasks".to_string(),
            columns: vec![
                KanbanColumn { title: "todo".to_string(), items: vec![] },
                KanbanColumn { title: "active".to_string(), items: vec![] },
                KanbanColumn { title: "blocked".to_string(), items: vec![] },
                KanbanColumn { title: "completed".to_string(), items: vec![] },
            ],
        }
    }
}