use serde::{Deserialize, Serialize};

/// Represents a warrant that grants a subject a relation on a resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warrant {
    /// The type of the resource.
    pub resource_type: String,

    /// The unique identifier of the resource.
    pub resource_id: String,

    /// The relation granted to the subject.
    pub relation: String,

    /// The subject that is granted the relation.
    pub subject: Subject,
}

/// Represents a subject in a warrant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    /// The type of the subject.
    pub resource_type: String,

    /// The unique identifier of the subject.
    pub resource_id: String,
}