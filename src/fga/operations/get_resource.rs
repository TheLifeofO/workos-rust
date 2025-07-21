use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{Resource, Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`GetResource`].
#[derive(Debug, Serialize)]
pub struct GetResourceParams<'a> {
    /// The type of the resource.
    pub resource_type: &'a str,

    /// The unique identifier of the resource.
    pub resource_id: &'a str,
}

/// An error returned from [`GetResource`].
#[derive(Debug, Error)]
pub enum GetResourceError {}

impl From<GetResourceError> for WorkOsError<GetResourceError> {
    fn from(err: GetResourceError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Get a resource](https://workos.com/docs/reference/fga/resource/get)
#[async_trait]
pub trait GetResource {
    /// Retrieves an existing resource by its type and ID.
    ///
    /// [WorkOS Docs: Get a resource](https://workos.com/docs/reference/fga/resource/get)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), GetResourceError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let resource = workos
    ///     .fga()
    ///     .get_resource(&GetResourceParams {
    ///         resource_type: "document",
    ///         resource_id: "doc_123",
    ///     })
    ///     .await?;
    ///
    /// println!("Resource: {:?}", resource);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_resource(
        &self,
        params: &GetResourceParams<'_>,
    ) -> WorkOsResult<Resource, GetResourceError>;
}

#[async_trait]
impl GetResource for Fga<'_> {
    async fn get_resource(
        &self,
        params: &GetResourceParams<'_>,
    ) -> WorkOsResult<Resource, GetResourceError> {
        let url = self
            .workos
            .base_url()
            .join(&format!(
                "/fga/v1/resources/{}/{}",
                params.resource_type, params.resource_id
            ))?;
        let resource = self
            .workos
            .client()
            .get(url)
            .bearer_auth(self.workos.key())
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
    async fn it_calls_the_get_resource_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/resources/document/doc_123")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "type": "document",
                    "id": "doc_123",
                    "metadata": {}
                })
                .to_string(),
            )
            .create_async()
            .await;

        let resource = workos
            .fga()
            .get_resource(&GetResourceParams {
                resource_type: "document",
                resource_id: "doc_123",
            })
            .await
            .unwrap();

        assert_eq!(resource.resource_type, "document");
        assert_eq!(resource.resource_id, "doc_123");
    }
}