use serde::{Deserialize, Serialize};
use crate::fga::{Resource, Warrant};

/// The Query Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    /// The resource type
    #[serde(flatten)]
    pub resource: Resource,

    /// The relation
    pub relation: String,

    /// The warrant used to query the resource.
    pub warrant: Warrant,

    /// The subjects that have access to the resource.
    pub is_implicit: bool,

    /// Any meta stored on the resource.
    pub meta: Option<serde_json::Value>,
}