use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::Fga;
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`DeleteResourceType`].
#[derive(Debug, Serialize)]
pub struct DeleteResourceTypeParams<'a> {
    /// The name of the resource type to delete.
    pub resource_type: &'a str,
}

/// An error returned from [`DeleteResourceType`].
#[derive(Debug, Error)]
pub enum DeleteResourceTypeError {}

impl From<DeleteResourceTypeError> for WorkOsError<DeleteResourceTypeError> {
    fn from(err: DeleteResourceTypeError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Delete a resource type](https://workos.com/docs/reference/fga/resource-type/delete)
#[async_trait]
pub trait DeleteResourceType {
    /// Permanently deletes the specified resource-type definition.
    ///
    /// **Warning**: this is irreversible.
    ///
    /// [WorkOS Docs: Delete a resource type](https://workos.com/docs/reference/fga/resource-type/delete)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), DeleteResourceTypeError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// workos
    ///     .fga()
    ///     .delete_resource_type(&DeleteResourceTypeParams {
    ///         resource_type: "spreadsheet",
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_resource_type(
        &self,
        params: &DeleteResourceTypeParams<'_>,
    ) -> WorkOsResult<(), DeleteResourceTypeError>;
}

#[async_trait]
impl DeleteResourceType for Fga<'_> {
    async fn delete_resource_type(
        &self,
        params: &DeleteResourceTypeParams<'_>,
    ) -> WorkOsResult<(), DeleteResourceTypeError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/fga/v1/resource-types/{}", params.resource_type))?;

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
    async fn it_calls_the_delete_resource_type_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("DELETE", "/fga/v1/resource-types/spreadsheet")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(204)
            .create_async()
            .await;

        let result = workos
            .fga()
            .delete_resource_type(&DeleteResourceTypeParams {
                resource_type: "spreadsheet",
            })
            .await;

        assert_matches!(result, Ok(()));
    }
}