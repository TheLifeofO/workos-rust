use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{ResourceType, Fga, RelationRule};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`UpdateResourceType`].
#[derive(Debug, Serialize)]
pub struct UpdateResourceTypeParams<'a> {
    /// The resource-type name to update (path parameter).
    #[serde(skip)]
    pub resource_type: &'a str,

    /// Map of relation-name → relation-rule.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub relations: &'a std::collections::HashMap<String, RelationRule>,
}

/// An error returned from [`UpdateResourceType`].
#[derive(Debug, Error)]
pub enum UpdateResourceTypeError {}

impl From<UpdateResourceTypeError> for WorkOsError<UpdateResourceTypeError> {
    fn from(err: UpdateResourceTypeError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Update a resource type](https://workos.com/docs/reference/fga/resource-type/update)
#[async_trait]
pub trait UpdateResourceType {
    /// Updates the definition of an existing resource type.
    ///
    /// **Note**: this performs a **full replace** of the relations map.
    ///
    /// [WorkOS Docs: Update a resource type](https://workos.com/docs/reference/fga/resource-type/update)
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
    /// # async fn run() -> WorkOsResult<(), UpdateResourceTypeError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let mut relations = HashMap::new();
    /// relations.insert("owner".into(), RelationRule::This { this: serde_json::json!({}) });
    /// relations.insert("reader".into(), RelationRule::This { this: serde_json::json!({}) });
    ///
    /// let updated = workos
    ///     .fga()
    ///     .update_resource_type(&UpdateResourceTypeParams {
    ///         resource_type: "document",
    ///         relations: &relations,
    ///     })
    ///     .await?;
    ///
    /// assert_eq!(updated.r#type, "document");
    /// # Ok(())
    /// # }
    /// ```
    async fn update_resource_type(
        &self,
        params: &UpdateResourceTypeParams<'_>,
    ) -> WorkOsResult<ResourceType, UpdateResourceTypeError>;
}

#[async_trait]
impl UpdateResourceType for Fga<'_> {
    async fn update_resource_type(
        &self,
        params: &UpdateResourceTypeParams<'_>,
    ) -> WorkOsResult<ResourceType, UpdateResourceTypeError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/fga/v1/resource-types/{}", params.resource_type))?;

        let resource_type = self
            .workos
            .client()
            .put(url)
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
    async fn it_calls_the_update_resource_type_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("PUT", "/fga/v1/resource-types/document")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "type": "document",
                    "relations": {
                        "owner": { "this": {} },
                        "reader": { "this": {} }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let mut relations = HashMap::new();
        relations.insert(
            "owner".into(),
            RelationRule::This {
                this: serde_json::Value::Null,
            },
        );
        relations.insert(
            "reader".into(),
            RelationRule::This {
                this: serde_json::Value::Null,
            },
        );

        let resource_type = workos
            .fga()
            .update_resource_type(&UpdateResourceTypeParams {
                resource_type: "document",
                relations: &relations,
            })
            .await
            .unwrap();

        assert_eq!(resource_type.r#type, "document");
        assert!(resource_type.relations.contains_key("reader"));
    }
}