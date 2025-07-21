use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{ResourceType, Fga, RelationRule};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`CreateResourceType`].
#[derive(Debug, Serialize)]
pub struct CreateResourceTypeParams<'a> {
    /// The unique name of the new resource type, e.g. `"spreadsheet"`.
    #[serde(rename = "type")]
    pub resource_type: &'a str,

    /// Map of relation-name → relation-rule.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub relations: &'a std::collections::HashMap<String, RelationRule>,
}

/// An error returned from [`CreateResourceType`].
#[derive(Debug, Error)]
pub enum CreateResourceTypeError {}

impl From<CreateResourceTypeError> for WorkOsError<CreateResourceTypeError> {
    fn from(err: CreateResourceTypeError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Create a resource type](https://workos.com/docs/reference/fga/resource-type/create)
#[async_trait]
pub trait CreateResourceType {
    /// Creates a new resource-type definition in the current environment.
    ///
    /// [WorkOS Docs: Create a resource type](https://workos.com/docs/reference/fga/resource-type/create)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), CreateResourceTypeError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let mut relations = HashMap::new();
    /// relations.insert("editor".into(), RelationRule::This { this: serde_json::json!({}) });
    /// relations.insert("viewer".into(), RelationRule::This { this: serde_json::json!({}) });
    ///
    /// let rt = workos
    ///     .fga()
    ///     .create_resource_type(&CreateResourceTypeParams {
    ///         resource_type: "spreadsheet",
    ///         relations: &relations,
    ///     })
    ///     .await?;
    ///
    /// println!("Created resource type: {}", rt.r#type);
    /// # Ok(())
    /// # }
    /// ```
    async fn create_resource_type(
        &self,
        params: &CreateResourceTypeParams<'_>,
    ) -> WorkOsResult<ResourceType, CreateResourceTypeError>;
}

#[async_trait]
impl CreateResourceType for Fga<'_> {
    async fn create_resource_type(
        &self,
        params: &CreateResourceTypeParams<'_>,
    ) -> WorkOsResult<ResourceType, CreateResourceTypeError> {
        let url = self.workos.base_url().join("/fga/v1/resource-types")?;
        let resource_type = self
            .workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&params)
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<ResourceType>()
            .await?;

        Ok(resource_type)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use std::collections::HashMap;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};
    use crate::fga::RelationRule;

    #[tokio::test]
    async fn it_calls_the_create_resource_type_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/resource-types")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(201)
            .with_body(
                json!({
                    "type": "spreadsheet",
                    "relations": {
                        "editor": { "this": {} },
                        "viewer": { "this": {} }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let mut relations = HashMap::new();
        relations.insert(
            "editor".into(),
            RelationRule::This {
                this: serde_json::Value::Null,
            },
        );
        relations.insert(
            "viewer".into(),
            RelationRule::This {
                this: serde_json::Value::Null,
            },
        );

        let resource_type = workos
            .fga()
            .create_resource_type(&CreateResourceTypeParams {
                resource_type: "spreadsheet",
                relations: &relations,
            })
            .await
            .unwrap();

        assert_eq!(resource_type.r#type, "spreadsheet");
    }
}