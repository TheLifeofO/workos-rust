use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{Resource, Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`UpdateResource`].
#[derive(Debug, Serialize)]
pub struct UpdateResourceParams<'a> {
    /// The type of the resource.
    #[serde(rename = "type")]
    pub r#type: &'a str,

    /// The unique identifier of the resource.
    pub id: &'a str,

    /// Optional metadata to update for the resource.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: &'a std::collections::HashMap<String, serde_json::Value>,
}

/// An error returned from [`UpdateResource`].
#[derive(Debug, Error)]
pub enum UpdateResourceError {}

impl From<UpdateResourceError> for WorkOsError<UpdateResourceError> {
    fn from(err: UpdateResourceError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Update a resource](https://workos.com/docs/reference/fga/resource/update)
#[async_trait]
pub trait UpdateResource {
    /// Updates an existing resource in the current environment.
    ///
    /// [WorkOS Docs: Update a resource](https://workos.com/docs/reference/fga/resource/update)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    /// use std::collections::HashMap;
    ///
    /// # async fn run() -> WorkOsResult<(), UpdateResourceError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let mut metadata = HashMap::new();
    /// metadata.insert("author".into(), serde_json::json!("Marcelina Davis"));
    ///
    /// let resource = workos
    ///     .fga()
    ///     .update_resource(&UpdateResourceParams {
    ///         r#type: "document",
    ///         id: "doc_123",
    ///         metadata: &metadata,
    ///     })
    ///     .await?;
    ///
    /// println!("Updated resource: {:?}", resource);
    /// # Ok(())
    /// # }
    /// ```
    async fn update_resource(
        &self,
        params: &UpdateResourceParams<'_>,
    ) -> WorkOsResult<Resource, UpdateResourceError>;
}

#[async_trait]
impl UpdateResource for Fga<'_> {
    async fn update_resource(
        &self,
        params: &UpdateResourceParams<'_>,
    ) -> WorkOsResult<Resource, UpdateResourceError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/fga/v1/resources/{}/{}", params.r#type, params.id))?;
        let resource = self
            .workos
            .client()
            .put(url)
            .bearer_auth(self.workos.key())
            .json(&serde_json::json!({
                "metadata": params.metadata
            }))
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Resource>()
            .await?;

        Ok(resource)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_update_resource_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("PUT", "/fga/v1/resources/document/doc_123")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "type": "document",
                    "id": "doc_123",
                    "metadata": { "author": "Marcelina Davis" }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("author".into(), json!("Marcelina Davis"));

        let resource = workos
            .fga()
            .update_resource(&UpdateResourceParams {
                r#type: "document",
                id: "doc_123",
                metadata: &metadata,
            })
            .await
            .unwrap();

        assert_eq!(resource.resource_type, "document");
        assert_eq!(resource.resource_id, "doc_123");
    }
}