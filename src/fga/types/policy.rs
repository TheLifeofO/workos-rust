use serde::{Deserialize, Serialize};

/// Represents a policy that defines access control rules for your application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// The name of the policy.
    pub name: String,

    /// The description of the policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The language of the policy (e.g., "expr").
    pub language: String,

    /// The parameters of the policy.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<PolicyParameter>,

    /// The policy expression.
    pub expression: String,
}

/// A parameter of a policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyParameter {
    /// The name of the parameter.
    pub name: String,

    /// The type of the parameter.
    pub r#type: String,
}