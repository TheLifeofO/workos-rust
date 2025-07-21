use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: String,
    pub resource_id: String,
}