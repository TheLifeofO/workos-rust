use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{Resource, Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`BatchWriteResources`].
#[derive(Debug, Serialize)]
pub struct BatchWriteResourcesParams<'a> {
    /// List of resources to create or delete.
    pub writes: &'a [ResourceWrite<'a>],
}

/// A single resource write operation.
#[derive(Debug, Serialize)]
pub struct ResourceWrite<'a> {
    /// The type of the resource.
    #[serde(rename = "type")]
    pub r#type: &'a str,

    /// The unique identifier of the resource.
    pub id: &'a str,

    /// Optional metadata associated with the resource.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: &'a std::collections::HashMap<String, serde_json::Value>,

    /// Whether to create (`true`) or delete (`false`) the resource.
    pub create: bool,
}

/// An error returned from [`BatchWriteResources`].
#[derive(Debug, Error)]
#[derive(PartialEq)]
pub enum BatchWriteResourcesError {}

impl From<BatchWriteResourcesError> for WorkOsError<BatchWriteResourcesError> {
    fn from(err: BatchWriteResourcesError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Batch Write Resources](https://workos.com/docs/reference/fga/resource/batch-write)
#[async_trait]
pub trait BatchWriteResources {
    /// Executes a batch of resource writes in the current environment.
    ///
    /// [WorkOS Docs: Batch Write Resources](https://workos.com/docs/reference/fga/resource/batch-write)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    /// use std::collections::HashMap;
    ///
    /// # async fn run() -> WorkOsResult<(), BatchWriteResourcesError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let writes = vec![
    ///     ResourceWrite {
    ///         r#type: "document",
    ///         id: "doc_123",
    ///         metadata: &HashMap::new(),
    ///         create: true,
    ///     },
    ///     ResourceWrite {
    ///         r#type: "document",
    ///         id: "doc_456",
    ///         metadata: &HashMap::new(),
    ///         create: false,
    ///     },
    /// ];
    ///
    /// workos
    ///     .fga()
    ///     .batch_write_resources(&BatchWriteResourcesParams { writes: &writes })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn batch_write_resources(
        &self,
        params: &BatchWriteResourcesParams<'_>,
    ) -> WorkOsResult<(), BatchWriteResourcesError>;
}

#[async_trait]
impl BatchWriteResources for Fga<'_> {
    async fn batch_write_resources(
        &self,
        params: &BatchWriteResourcesParams<'_>,
    ) -> WorkOsResult<(), BatchWriteResourcesError> {
        let url = self.workos.base_url().join("/fga/v1/resources/batch")?;
        self.workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&serde_json::json!({
                "writes": params.writes
            }))
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_batch_write_resources_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/resources/batch")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(204)
            .create_async()
            .await;

        let context = std::collections::HashMap::new();

        let writes = vec![
            ResourceWrite {
                r#type: "document",
                id: "doc_123",
                metadata: &context,
                create: true,
            },
            ResourceWrite {
                r#type: "document",
                id: "doc_456",
                metadata: &context,
                create: false,
            },
        ];

        let result = workos
            .fga()
            .batch_write_resources(&BatchWriteResourcesParams { writes: &writes })
            .await;

        assert_eq!(result.is_ok(), true);
    }
}