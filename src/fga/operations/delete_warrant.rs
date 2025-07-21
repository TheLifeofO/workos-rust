use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{Subject, Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`DeleteWarrant`].
#[derive(Debug, Serialize)]
pub struct DeleteWarrantParams<'a> {
    /// The type of the resource.
    pub resource_type: &'a str,

    /// The unique identifier of the resource.
    pub resource_id: &'a str,

    /// The relation to revoke.
    pub relation: &'a str,

    /// The subject to revoke the relation from.
    pub subject: Subject,
}

/// An error returned from [`DeleteWarrant`].
#[derive(Debug, Error)]
pub enum DeleteWarrantError {}

impl From<DeleteWarrantError> for WorkOsError<DeleteWarrantError> {
    fn from(err: DeleteWarrantError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Delete a warrant](https://workos.com/docs/reference/fga/warrant/delete)
#[async_trait]
pub trait DeleteWarrant {
    /// Deletes (revokes) a warrant in the current environment.
    ///
    /// [WorkOS Docs: Delete a warrant](https://workos.com/docs/reference/fga/warrant/delete)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), DeleteWarrantError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// workos
    ///     .fga()
    ///     .delete_warrant(&DeleteWarrantParams {
    ///         resource_type: "document",
    ///         resource_id: "doc_123",
    ///         relation: "viewer",
    ///         subject: Subject {
    ///             resource_type: String::from("user"),
    ///             resource_id: String::from("user_123"),
    ///         },
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_warrant(
        &self,
        params: &DeleteWarrantParams<'_>,
    ) -> WorkOsResult<(), DeleteWarrantError>;
}

#[async_trait]
impl DeleteWarrant for Fga<'_> {
    async fn delete_warrant(
        &self,
        params: &DeleteWarrantParams<'_>,
    ) -> WorkOsResult<(), DeleteWarrantError> {
        let url = self.workos.base_url().join("/fga/v1/warrants")?;
        self.workos
            .client()
            .delete(url)
            .bearer_auth(self.workos.key())
            .json(&params)
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
    async fn it_calls_the_delete_warrant_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("DELETE", "/fga/v1/warrants")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(204)
            .create_async()
            .await;

        let result = workos
            .fga()
            .delete_warrant(&DeleteWarrantParams {
                resource_type: "document",
                resource_id: "doc_123",
                relation: "viewer",
                subject: Subject {
                    resource_type: "user".parse().unwrap(),
                    resource_id: "user_123".parse().unwrap(),
                },
            })
            .await;

        assert_matches!(result, Ok(()));
    }
}