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
    Adr,
}

#[derive(Debug, PartialEq)]
pub enum AppState {
    Normal,
    CreatingDocument,
    CreatingChildDocument,
    CreatingAdr,
    EditingContent,
    Confirming,
}


impl KanbanBoard {
    pub fn create_strategy_board() -> Self {
        Self {
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
            columns: vec![
                KanbanColumn { title: "todo".to_string(), items: vec![] },
                KanbanColumn { title: "active".to_string(), items: vec![] },
                KanbanColumn { title: "blocked".to_string(), items: vec![] },
                KanbanColumn { title: "completed".to_string(), items: vec![] },
            ],
        }
    }

    pub fn create_adr_board() -> Self {
        Self {
            columns: vec![
                KanbanColumn { title: "draft".to_string(), items: vec![] },
                KanbanColumn { title: "discussion".to_string(), items: vec![] },
                KanbanColumn { title: "decided".to_string(), items: vec![] },
                KanbanColumn { title: "superseded".to_string(), items: vec![] },
            ],
        }
    }
}