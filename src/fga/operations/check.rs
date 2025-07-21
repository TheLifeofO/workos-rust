use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`Check`].
#[derive(Debug, Serialize)]
pub struct CheckParams<'a> {
    /// The subject that is requesting access.
    pub subject: &'a str,

    /// The relation to check.
    pub relation: &'a str,

    /// The resource to check access against.
    pub resource: &'a str,
}

/// An error returned from [`Check`].
#[derive(Debug, Error)]
pub enum CheckError {
    /// Not allowed error, when the subject does not have the relation on the resource.
    #[error("Not allowed: subject does not have the relation on the resource")]
    NotAllowed,
}

impl From<CheckError> for WorkOsError<CheckError> {
    fn from(err: CheckError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Check](https://workos.com/docs/reference/fga/check)
#[async_trait]
pub trait Check {
    /// Checks if a subject has a particular relation on a resource.
    ///
    /// [WorkOS Docs: Check](https://workos.com/docs/reference/fga/check)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), CheckError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let result = workos
    ///     .fga()
    ///     .check(&CheckParams {
    ///         subject: "user_123",
    ///         relation: "viewer",
    ///         resource: "document:doc_123",
    ///     })
    ///     .await?;
    ///
    /// println!("Check result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    async fn check(
        &self,
        params: &CheckParams<'_>,
    ) -> WorkOsResult<bool, CheckError>;
}

#[async_trait]
impl Check for Fga<'_> {
    async fn check(
        &self,
        params: &CheckParams<'_>,
    ) -> WorkOsResult<bool, CheckError> {
        let url = self.workos.base_url().join("/fga/v1/check")?;
        let response = self
            .workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&params)
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?;

        let result: serde_json::Value = response.json().await?;
        if let Some(allowed) = result.get("allowed").and_then(|v| v.as_bool()) {
            Ok(allowed)
        } else {
            Err(CheckError::NotAllowed.into())
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_check_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/check")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "allowed": true
                })
                .to_string(),
            )
            .create_async()
            .await;

        let result = workos
            .fga()
            .check(&CheckParams {
                subject: "user_123",
                relation: "viewer",
                resource: "document:doc_123",
            })
            .await
            .unwrap();

        assert!(result);
    }
}