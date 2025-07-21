use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{ Fga, Subject, Resource, CreateWarrantParams};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`BatchWriteWarrants`].
#[derive(Debug, Serialize)]
pub struct BatchWriteWarrantsParams<'a> {
    /// List of warrants to create or delete.
    pub writes: &'a Vec<CreateWarrantParams<'a>>,
}

/// An error returned from [`BatchWriteWarrants`].
#[derive(Debug, Error)]
#[derive(PartialEq)]
pub enum BatchWriteWarrantsError {}

impl From<BatchWriteWarrantsError> for WorkOsError<BatchWriteWarrantsError> {
    fn from(err: BatchWriteWarrantsError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Batch Write Warrants](https://workos.com/docs/reference/fga/warrant/batch-write)
#[async_trait]
pub trait BatchWriteWarrants {
    /// Executes a batch of warrant writes in the current environment.
    ///
    /// [WorkOS Docs: Batch Write Warrants](https://workos.com/docs/reference/fga/warrant/batch-write)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    /// use std::collections::HashMap;
    ///
    /// # async fn run() -> WorkOsResult<(), BatchWriteWarrantsError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let writes = vec![
    ///     CreateWarrantParams {
    ///        resource_type: "document",
    ///        resource_id: "doc_123",
    ///        relation: "viewer",
    ///        subject: Subject {
    ///          resource_type: "".to_string(),
    ///          resource_id: "".to_string(),},
    ///       policy: None,
    ///   },
    /// ];
    ///
    /// workos
    ///     .fga()
    ///     .batch_write_warrants(&BatchWriteWarrantsParams { writes: &writes })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn batch_write_warrants(
        &self,
        params: &BatchWriteWarrantsParams<'_>,
    ) -> WorkOsResult<(), BatchWriteWarrantsError>;
}

#[async_trait]
impl BatchWriteWarrants for Fga<'_> {
    async fn batch_write_warrants(
        &self,
        params: &BatchWriteWarrantsParams<'_>,
    ) -> WorkOsResult<(), BatchWriteWarrantsError> {
        let url = self.workos.base_url().join("/fga/v1/warrants/batch")?;
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
    use crate::fga::{Resource, Subject};

    #[tokio::test]
    async fn it_calls_the_batch_write_warrants_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/warrants/batch")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(204)
            .create_async()
            .await;
        
        let writes = vec![
            CreateWarrantParams {
                resource_type: "document",
                resource_id: "doc_123",
                relation: "viewer",
                subject: Subject {
                    resource_type: "user".to_string(),
                    resource_id: "user_456".to_string(),
                },
                policy: None,
            },
            CreateWarrantParams {
                resource_type: "document",
                resource_id: "doc_789",
                relation: "editor",
                subject: Subject {
                    resource_type: "user".to_string(),
                    resource_id: "user_101".to_string(),
                },
                policy: None,
            },
        ];

        let result = workos
            .fga()
            .batch_write_warrants(&BatchWriteWarrantsParams { writes: &writes })
            .await;

        assert_eq!(result.is_ok(), true);
    }
}