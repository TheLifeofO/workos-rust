use async_trait::async_trait;
use serde::{Serialize};
use thiserror::Error;

use crate::fga::{Resource, Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`CreateResource`].
#[derive(Debug, Serialize)]
pub struct CreateResourceParams<'a> {
    /// The type of the resource.
    pub resource_type: &'a str,

    /// The unique identifier of the resource.
    pub resource_id: &'a str,
}

/// An error returned from [`CreateResource`].
#[derive(Debug, Error)]
pub enum CreateResourceError {}

impl From<CreateResourceError> for WorkOsError<CreateResourceError> {
    fn from(err: CreateResourceError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Create a resource](https://workos.com/docs/reference/fga/resource/create)
#[async_trait]
pub trait CreateResource {
    /// Creates a new resource in the current environment.
    ///
    /// [WorkOS Docs: Create a resource](https://workos.com/docs/reference/fga/resource/create)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    /// use std::collections::HashMap;
    ///
    /// # async fn run() -> WorkOsResult<(), CreateResourceError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let mut metadata = HashMap::new();
    /// metadata.insert("author".into(), serde_json::json!("Marcelina Davis"));
    ///
    /// let resource = workos
    ///     .fga()
    ///     .create_resource(&CreateResourceParams {
    ///         resource_type: "document",
    ///         resource_id: "doc_123",
    ///     })
    ///     .await?;
    ///
    /// println!("Created resource: {:?}", resource);
    /// # Ok(())
    /// # }
    /// ```
    async fn create_resource(
        &self,
        params: &CreateResourceParams<'_>,
    ) -> WorkOsResult<Resource, CreateResourceError>;
}

#[async_trait]
impl CreateResource for Fga<'_> {
    async fn create_resource(
        &self,
        params: &CreateResourceParams<'_>,
    ) -> WorkOsResult<Resource, CreateResourceError> {
        let url = self.workos.base_url().join("/fga/v1/resources")?;
        let resource = self
            .workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&params)
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
    async fn it_calls_the_create_resource_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/resources")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(201)
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

        let resource = workos
            .fga()
            .create_resource(&CreateResourceParams {
                resource_type: "document",
                resource_id: "doc_123",
            })
            .await
            .unwrap();

        assert_eq!(resource.resource_id, "document");
        assert_eq!(resource.resource_type, "doc_123");
    }
}