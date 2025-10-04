use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Maximum length for document IDs
const MAX_ID_LENGTH: usize = 35;

/// Document identifier - always derived from title as a slug
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(String);

impl DocumentId {
    /// Create a new DocumentId from a raw string (used for ADRs with custom format)
    pub fn new(id: &str) -> Self {
        let capped_id = if id.chars().count() > MAX_ID_LENGTH {
            // Use char-aware truncation to avoid cutting UTF-8 characters
            id.chars().take(MAX_ID_LENGTH).collect::<String>()
        } else {
            id.to_string()
        };
        Self(capped_id)
    }

    /// Create a DocumentId from a title by converting to slug
    pub fn from_title(title: &str) -> Self {
        let slug = Self::title_to_slug(title);
        Self::new(&slug)
    }

    /// Convert title to URL-friendly slug
    pub fn title_to_slug(title: &str) -> String {
        let slug = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        // Cap the length and ensure we don't cut off in the middle of a word
        if slug.chars().count() > MAX_ID_LENGTH {
            // Use char-aware truncation to avoid cutting UTF-8 characters
            let truncated: String = slug.chars().take(MAX_ID_LENGTH).collect();
            // Find the last dash to avoid cutting in the middle of a word
            if let Some(last_dash) = truncated.rfind('-') {
                if last_dash > MAX_ID_LENGTH / 2 {
                    // Only use the dash if it's not too early in the string
                    return truncated[..last_dash].to_string();
                }
            }
            truncated
        } else {
            slug
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DocumentId {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<&str> for DocumentId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Parent reference for documents in flexible flight levels
/// Handles the case where intermediate levels may be optional
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParentReference {
    /// Document has a specific parent
    Some(DocumentId),
    /// Document has no parent (top-level like Vision or ADR)
    None,
    /// Document would have a parent but that level is disabled in configuration
    /// Used for path construction with "NULL" string
    Null,
}

impl ParentReference {
    /// Convert to string for path construction
    pub fn to_path_string(&self) -> String {
        match self {
            ParentReference::Some(id) => id.as_str().to_string(),
            ParentReference::None => "vision".to_string(), // Default to vision for top-level
            ParentReference::Null => "NULL".to_string(),
        }
    }

    /// Check if this reference points to an actual parent
    pub fn has_parent(&self) -> bool {
        matches!(self, ParentReference::Some(_))
    }

    /// Get the parent ID if it exists
    pub fn parent_id(&self) -> Option<&DocumentId> {
        match self {
            ParentReference::Some(id) => Some(id),
            _ => None,
        }
    }

    /// Create from optional document ID
    pub fn from_option(id: Option<DocumentId>) -> Self {
        match id {
            Some(id) => ParentReference::Some(id),
            None => ParentReference::None,
        }
    }

    /// Create a null reference for disabled levels
    pub fn null() -> Self {
        ParentReference::Null
    }
}

impl From<DocumentId> for ParentReference {
    fn from(id: DocumentId) -> Self {
        ParentReference::Some(id)
    }
}

impl From<Option<DocumentId>> for ParentReference {
    fn from(opt: Option<DocumentId>) -> Self {
        ParentReference::from_option(opt)
    }
}

impl fmt::Display for ParentReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_path_string())
    }
}

/// Document type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    Vision,
    Strategy,
    Initiative,
    Task,
    Adr,
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DocumentType::Vision => write!(f, "vision"),
            DocumentType::Strategy => write!(f, "strategy"),
            DocumentType::Initiative => write!(f, "initiative"),
            DocumentType::Task => write!(f, "task"),
            DocumentType::Adr => write!(f, "adr"),
        }
    }
}

impl FromStr for DocumentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vision" => Ok(DocumentType::Vision),
            "strategy" => Ok(DocumentType::Strategy),
            "initiative" => Ok(DocumentType::Initiative),
            "task" => Ok(DocumentType::Task),
            "adr" => Ok(DocumentType::Adr),
            _ => Err(format!("Unknown document type: {}", s)),
        }
    }
}

/// Document phase/status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    // Vision phases
    Draft,
    Review,
    Published,

    // ADR phases
    Discussion,
    Decided,
    Superseded,

    // General phases
    Backlog,
    Todo,
    Active,
    Blocked,
    Completed,

    // Strategy/Initiative phases
    Shaping,
    Design,
    Ready,
    Decompose,
    Discovery,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Phase::Draft => write!(f, "draft"),
            Phase::Review => write!(f, "review"),
            Phase::Published => write!(f, "published"),
            Phase::Discussion => write!(f, "discussion"),
            Phase::Decided => write!(f, "decided"),
            Phase::Superseded => write!(f, "superseded"),
            Phase::Backlog => write!(f, "backlog"),
            Phase::Todo => write!(f, "todo"),
            Phase::Active => write!(f, "active"),
            Phase::Blocked => write!(f, "blocked"),
            Phase::Completed => write!(f, "completed"),
            Phase::Shaping => write!(f, "shaping"),
            Phase::Design => write!(f, "design"),
            Phase::Ready => write!(f, "ready"),
            Phase::Decompose => write!(f, "decompose"),
            Phase::Discovery => write!(f, "discovery"),
        }
    }
}

/// Document tag that can be either a phase or a string
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tag {
    Phase(Phase),
    Label(String),
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tag::Phase(phase) => write!(f, "#phase/{}", phase),
            Tag::Label(label) => {
                if label.starts_with('#') {
                    write!(f, "{}", label)
                } else {
                    write!(f, "#{}", label)
                }
            }
        }
    }
}

impl From<Phase> for Tag {
    fn from(phase: Phase) -> Self {
        Tag::Phase(phase)
    }
}

impl From<String> for Tag {
    fn from(label: String) -> Self {
        Tag::Label(label)
    }
}

impl From<&str> for Tag {
    fn from(label: &str) -> Self {
        Tag::Label(label.to_string())
    }
}

impl std::str::FromStr for Tag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(phase_str) = s.strip_prefix("#phase/") {
            // Remove "#phase/"
            match phase_str {
                "draft" => Ok(Tag::Phase(Phase::Draft)),
                "review" => Ok(Tag::Phase(Phase::Review)),
                "published" => Ok(Tag::Phase(Phase::Published)),
                "discussion" => Ok(Tag::Phase(Phase::Discussion)),
                "decided" => Ok(Tag::Phase(Phase::Decided)),
                "superseded" => Ok(Tag::Phase(Phase::Superseded)),
                "todo" => Ok(Tag::Phase(Phase::Todo)),
                "active" => Ok(Tag::Phase(Phase::Active)),
                "blocked" => Ok(Tag::Phase(Phase::Blocked)),
                "completed" => Ok(Tag::Phase(Phase::Completed)),
                "shaping" => Ok(Tag::Phase(Phase::Shaping)),
                "design" => Ok(Tag::Phase(Phase::Design)),
                "ready" => Ok(Tag::Phase(Phase::Ready)),
                "decompose" => Ok(Tag::Phase(Phase::Decompose)),
                "discovery" => Ok(Tag::Phase(Phase::Discovery)),
                "backlog" => Ok(Tag::Phase(Phase::Backlog)),
                _ => Err(()), // Unknown phase
            }
        } else if let Some(stripped) = s.strip_prefix("#") {
            Ok(Tag::Label(stripped.to_string())) // Remove "#"
        } else {
            Ok(Tag::Label(s.to_string()))
        }
    }
}

impl Tag {
    /// Convert tag back to its string representation (reverse of from_str)
    pub fn to_str(&self) -> String {
        match self {
            Tag::Phase(phase) => format!("#phase/{}", phase),
            Tag::Label(label) => {
                if label.starts_with('#') {
                    label.clone()
                } else {
                    format!("#{}", label)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_to_slug() {
        assert_eq!(
            DocumentId::title_to_slug("Core Document Management Library"),
            "core-document-management-library"
        );
        assert_eq!(
            DocumentId::title_to_slug("ADR-001: Document Format"),
            "adr-001-document-format"
        );
        assert_eq!(
            DocumentId::title_to_slug("Storage & Indexing System"),
            "storage-indexing-system"
        );
        assert_eq!(
            DocumentId::title_to_slug("Multiple   Spaces   Between---Words"),
            "multiple-spaces-between-words"
        );
    }

    #[test]
    fn test_id_length_capping() {
        let very_long_title = "This is an extremely long title that should definitely exceed our maximum identifier length limit and needs to be truncated appropriately without breaking";
        let id = DocumentId::from_title(very_long_title);
        assert!(id.as_str().len() <= MAX_ID_LENGTH);
        assert!(!id.as_str().ends_with('-')); // Should not end with dash
    }

    #[test]
    fn test_adr_custom_id() {
        let id = DocumentId::new("001-my-architecture-decision");
        assert_eq!(id.as_str(), "001-my-architecture-decision");
    }

    #[test]
    fn test_tag_parsing() {
        assert_eq!(
            "#phase/draft".parse::<Tag>().unwrap(),
            Tag::Phase(Phase::Draft)
        );
        assert_eq!(
            "#phase/active".parse::<Tag>().unwrap(),
            Tag::Phase(Phase::Active)
        );
        assert_eq!(
            "#phase/discovery".parse::<Tag>().unwrap(),
            Tag::Phase(Phase::Discovery)
        );
        assert_eq!(
            "#vision".parse::<Tag>().unwrap(),
            Tag::Label("vision".to_string())
        );
        assert_eq!(
            "#strategy".parse::<Tag>().unwrap(),
            Tag::Label("strategy".to_string())
        );
        assert_eq!(
            "urgent".parse::<Tag>().unwrap(),
            Tag::Label("urgent".to_string())
        );
    }

    #[test]
    fn test_tag_to_str() {
        assert_eq!(Tag::Phase(Phase::Draft).to_str(), "#phase/draft");
        assert_eq!(Tag::Phase(Phase::Active).to_str(), "#phase/active");
        assert_eq!(Tag::Label("vision".to_string()).to_str(), "#vision");
        assert_eq!(
            Tag::Label("#already-has-hash".to_string()).to_str(),
            "#already-has-hash"
        );
    }

    #[test]
    fn test_tag_roundtrip() {
        let tags = vec![
            Tag::Phase(Phase::Draft),
            Tag::Phase(Phase::Completed),
            Tag::Label("urgent".to_string()),
            Tag::Label("vision".to_string()),
        ];

        for tag in tags {
            let str_repr = tag.to_str();
            let parsed_back = str_repr.parse::<Tag>().unwrap();
            assert_eq!(tag, parsed_back);
        }
    }
}
