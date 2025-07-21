use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::Fga;
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for a single check in [`BatchCheck`].
#[derive(Debug, Serialize)]
pub struct CheckTuple<'a> {
    /// The subject that is requesting access.
    pub subject: &'a str,

    /// The relation to check.
    pub relation: &'a str,

    /// The resource to check access against.
    pub resource: &'a str,
}

/// Parameters for [`BatchCheck`].
#[derive(Debug, Serialize)]
pub struct BatchCheckParams<'a> {
    /// List of check tuples to evaluate.
    pub checks: &'a [CheckTuple<'a>],
}

/// Result of a single check in a batch.
#[derive(Debug, Deserialize)]
pub struct CheckResult {
    /// The subject that was checked.
    pub subject: String,

    /// The relation that was checked.
    pub relation: String,

    /// The resource that was checked.
    pub resource: String,

    /// Whether the subject is allowed the relation on the resource.
    pub allowed: bool,
}

/// An error returned from [`BatchCheck`].
#[derive(Debug, Error)]
pub enum BatchCheckError {}

impl From<BatchCheckError> for WorkOsError<BatchCheckError> {
    fn from(err: BatchCheckError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Batch Check](https://workos.com/docs/reference/fga/check/batch)
#[async_trait]
pub trait BatchCheck {
    /// Executes a batch of checks and returns a list of results.
    ///
    /// [WorkOS Docs: Batch Check](https://workos.com/docs/reference/fga/check/batch)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), BatchCheckError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let checks = vec![
    ///     CheckTuple {
    ///         subject: "user_123",
    ///         relation: "viewer",
    ///         resource: "document:doc_123",
    ///     },
    ///     CheckTuple {
    ///         subject: "user_456",
    ///         relation: "editor",
    ///         resource: "document:doc_456",
    ///     },
    /// ];
    ///
    /// let results = workos
    ///     .fga()
    ///     .batch_check(&BatchCheckParams { checks: &checks })
    ///     .await?;
    ///
    /// for result in results {
    ///     println!("Check result: {:?}", result);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn batch_check(
        &self,
        params: &BatchCheckParams<'_>,
    ) -> WorkOsResult<Vec<CheckResult>, BatchCheckError>;
}

#[async_trait]
impl BatchCheck for Fga<'_> {
    async fn batch_check(
        &self,
        params: &BatchCheckParams<'_>,
    ) -> WorkOsResult<Vec<CheckResult>, BatchCheckError> {
        let url = self.workos.base_url().join("/fga/v1/check/batch")?;
        let results = self
            .workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&params)
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Vec<CheckResult>>()
            .await?;

        Ok(results)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_batch_check_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/check/batch")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!([
                    {
                        "subject": "user_123",
                        "relation": "viewer",
                        "resource": "document:doc_123",
                        "allowed": true
                    },
                    {
                        "subject": "user_456",
                        "relation": "editor",
                        "resource": "document:doc_456",
                        "allowed": false
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let checks = vec![
            CheckTuple {
                subject: "user_123",
                relation: "viewer",
                resource: "document:doc_123",
            },
            CheckTuple {
                subject: "user_456",
                relation: "editor",
                resource: "document:doc_456",
            },
        ];

        let results = workos
            .fga()
            .batch_check(&BatchCheckParams { checks: &checks })
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].allowed);
        assert!(!results[1].allowed);
    }
}