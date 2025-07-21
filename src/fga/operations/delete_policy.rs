use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::Fga;
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`DeletePolicy`].
#[derive(Debug, Serialize)]
pub struct DeletePolicyParams<'a> {
    /// The name of the policy to delete.
    pub name: &'a str,
}

/// An error returned from [`DeletePolicy`].
#[derive(Debug, Error)]
pub enum DeletePolicyError {}

impl From<DeletePolicyError> for WorkOsError<DeletePolicyError> {
    fn from(err: DeletePolicyError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Delete a policy](https://workos.com/docs/reference/fga/policy/delete)
#[async_trait]
pub trait DeletePolicy {
    /// Deletes an existing policy in the current environment.
    ///
    /// [WorkOS Docs: Delete a policy](https://workos.com/docs/reference/fga/policy/delete)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), DeletePolicyError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// workos
    ///     .fga()
    ///     .delete_policy(&DeletePolicyParams {
    ///         name: "ip_equal",
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_policy(
        &self,
        params: &DeletePolicyParams<'_>,
    ) -> WorkOsResult<(), DeletePolicyError>;
}

#[async_trait]
impl DeletePolicy for Fga<'_> {
    async fn delete_policy(
        &self,
        params: &DeletePolicyParams<'_>,
    ) -> WorkOsResult<(), DeletePolicyError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/fga/v1/policies/{}", params.name))?;
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
    async fn it_calls_the_delete_policy_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("DELETE", "/fga/v1/policies/ip_equal")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(204)
            .create_async()
            .await;

        let result = workos
            .fga()
            .delete_policy(&DeletePolicyParams {
                name: "ip_equal",
            })
            .await;

        assert_matches!(result, Ok(()));
    }
}