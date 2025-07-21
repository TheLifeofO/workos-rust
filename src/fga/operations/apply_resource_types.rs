use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{ResourceType, Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`ApplyResourceTypes`].
#[derive(Debug, Serialize)]
pub struct ApplyResourceTypesParams<'a> {
    /// **Complete** list of resource-type definitions to set for the environment.
    ///
    /// Any resource type **not** included will be **deleted**.
    pub resource_types: &'a [ResourceType],
}

/// An error returned from [`ApplyResourceTypes`].
#[derive(Debug, Error)]
pub enum ApplyResourceTypesError {}

impl From<ApplyResourceTypesError> for WorkOsError<ApplyResourceTypesError> {
    fn from(err: ApplyResourceTypesError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Apply resource types](https://workos.com/docs/reference/fga/resource-type/apply)
#[async_trait]
pub trait ApplyResourceTypes {
    /// **Replaces** the entire schema for the environment with the provided list.
    ///
    /// **Destructive operation** – existing types not in the payload are removed.
    ///
    /// [WorkOS Docs: Apply resource types](https://workos.com/docs/reference/fga/resource-type/apply)
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
    /// # async fn run() -> WorkOsResult<(), ApplyResourceTypesError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let doc = ResourceType {
    ///     r#type: "document".into(),
    ///     relations: {
    ///         let mut m = HashMap::new();
    ///         m.insert("owner".into(), RelationRule::This { this: serde_json::Value::Null });
    ///         m
    ///     },
    /// };
    ///
    /// workos
    ///     .fga()
    ///     .apply_resource_types(&ApplyResourceTypesParams {
    ///         resource_types: &[doc],
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn apply_resource_types(
        &self,
        params: &ApplyResourceTypesParams<'_>,
    ) -> WorkOsResult<Vec<ResourceType>, ApplyResourceTypesError>;
}

#[async_trait]
impl ApplyResourceTypes for Fga<'_> {
    async fn apply_resource_types(
        &self,
        params: &ApplyResourceTypesParams<'_>,
    ) -> WorkOsResult<Vec<ResourceType>, ApplyResourceTypesError> {
        let url = self.workos.base_url().join("/fga/v1/resource-types")?;
        let list = self
            .workos
            .client()
            .put(url)
            .bearer_auth(self.workos.key())
            .json(&serde_json::json!({
                "resource_types": params.resource_types
            }))
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Vec<ResourceType>>()
            .await?;

        Ok(list)
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
    async fn it_calls_the_apply_resource_types_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("PUT", "/fga/v1/resource-types")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!([
                    {
                        "type": "document",
                        "relations": { "owner": { "this": {} } }
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let doc = ResourceType {
            r#type: "document".into(),
            relations: {
                let mut m = HashMap::new();
                m.insert(
                    "owner".into(),
                    RelationRule::This {
                        this: serde_json::Value::Null,
                    },
                );
                m
            },
        };

        let result = workos
            .fga()
            .apply_resource_types(&ApplyResourceTypesParams {
                resource_types: &[doc],
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].r#type, "document");
    }
}