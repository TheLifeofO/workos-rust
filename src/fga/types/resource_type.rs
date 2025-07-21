use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A WorkOS FGA resource-type definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceType {
    /// The unique name of the resource type, e.g. `"document"`.
    #[serde(rename = "type")]
    pub r#type: String,

    /// Map of relation-name → relation-rule.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub relations: HashMap<String, RelationRule>,
}

/// Relation rule DSL.
///
/// The JSON can be one of three shapes: `this`, `inherit`, or `union`.
/// We flatten the enum so Serde chooses the right variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RelationRule {
    /// Grant directly attached to the resource.
    This { this: serde_json::Value },

    /// Inherit from another resource.
    Inherit { inherit: InheritRule },

    /// Union of multiple rules.
    Union { union: Vec<RelationRule> },
}

/// Inherit rule: `"relation"` from `"from"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InheritRule {
    /// The relation to inherit.
    pub relation: String,
    /// The source resource type.
    pub from: String,
}

/// Convenience alias used as a parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceTypeName(pub String);

impl From<&str> for ResourceTypeName {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}