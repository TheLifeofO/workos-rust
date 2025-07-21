use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::Fga;
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`DeleteResource`].
#[derive(Debug, Serialize)]
pub struct DeleteResourceParams<'a> {
    /// The type of the resource.
    pub resource_type: &'a str,

    /// The unique identifier of the resource.
    pub resource_id: &'a str,
}

/// An error returned from [`DeleteResource`].
#[derive(Debug, Error)]
pub enum DeleteResourceError {}

impl From<DeleteResourceError> for WorkOsError<DeleteResourceError> {
    fn from(err: DeleteResourceError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Delete a resource](https://workos.com/docs/reference/fga/resource/delete)
#[async_trait]
pub trait DeleteResource {
    /// Deletes an existing resource in the current environment.
    ///
    /// [WorkOS Docs: Delete a resource](https://workos.com/docs/reference/fga/resource/delete)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), DeleteResourceError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// workos
    ///     .fga()
    ///     .delete_resource(&DeleteResourceParams {
    ///         resource_type: "document",
    ///         resource_id: "doc_123",
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_resource(
        &self,
        params: &DeleteResourceParams<'_>,
    ) -> WorkOsResult<(), DeleteResourceError>;
}

#[async_trait]
impl DeleteResource for Fga<'_> {
    async fn delete_resource(
        &self,
        params: &DeleteResourceParams<'_>,
    ) -> WorkOsResult<(), DeleteResourceError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/fga/v1/resources/{}/{}", params.resource_type, params.resource_id))?;
        self.workos
            .client()
            .delete(url)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};
    use matches::assert_matches;

    #[tokio::test]
    async fn it_calls_the_delete_resource_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("DELETE", "/fga/v1/resources/document/doc_123")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(204)
            .create_async()
            .await;

        let result = workos
            .fga()
            .delete_resource(&DeleteResourceParams {
                resource_type: "document",
                resource_id: "doc_123",
            })
            .await;

        assert_matches!(result, Ok(()));
    }
}