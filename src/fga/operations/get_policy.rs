use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{Fga, Policy};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`GetPolicy`].
#[derive(Debug, Serialize)]
pub struct GetPolicyParams<'a> {
    /// The name of the policy to retrieve.
    pub name: &'a str,
}

/// An error returned from [`GetPolicy`].
#[derive(Debug, Error)]
pub enum GetPolicyError {}

impl From<GetPolicyError> for WorkOsError<GetPolicyError> {
    fn from(err: GetPolicyError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Get a policy](https://workos.com/docs/reference/fga/policy/get)
#[async_trait]
pub trait GetPolicy {
    /// Retrieves the definition of an existing policy.
    ///
    /// [WorkOS Docs: Get a policy](https://workos.com/docs/reference/fga/policy/get)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), GetPolicyError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let policy = workos
    ///     .fga()
    ///     .get_policy(&GetPolicyParams {
    ///         name: "ip_equal",
    ///     })
    ///     .await?;
    ///
    /// println!("Policy: {:?}", policy);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_policy(
        &self,
        params: &GetPolicyParams<'_>,
    ) -> WorkOsResult<Policy, GetPolicyError>;
}

#[async_trait]
impl GetPolicy for Fga<'_> {
    async fn get_policy(
        &self,
        params: &GetPolicyParams<'_>,
    ) -> WorkOsResult<Policy, GetPolicyError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/fga/v1/policies/{}", params.name))?;
        let policy = self
            .workos
            .client()
            .get(url)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Policy>()
            .await?;

        Ok(policy)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_get_policy_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/policies/ip_equal")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "name": "ip_equal",
                    "description": "Check if the client IP is in the range 192.168.x.x",
                    "language": "expr",
                    "parameters": [
                        {
                            "name": "clientIp",
                            "type": "string"
                        }
                    ],
                    "expression": "clientIp matches \"192\\\\.168\\\\..*\\\\..*\"",
                    "metadata": {}
                })
                .to_string(),
            )
            .create_async()
            .await;

        let policy = workos
            .fga()
            .get_policy(&GetPolicyParams {
                name: "ip_equal",
            })
            .await
            .unwrap();

        assert_eq!(policy.name, "ip_equal");
        assert_eq!(policy.description.unwrap(), "Check if the client IP is in the range 192.168.x.x");
        assert_eq!(policy.language, "expr");
        assert_eq!(policy.parameters.len(), 1);
        assert_eq!(policy.parameters[0].name, "clientIp");
        assert_eq!(policy.parameters[0].r#type, "string");
        assert_eq!(policy.expression, "clientIp matches \"192\\.168\\..*\\..*\"");
    }
}