use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{Policy, Fga, PolicyParameter};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`CreatePolicy`].
#[derive(Debug, Serialize)]
pub struct CreatePolicyParams<'a> {
    /// The name of the policy.
    pub name: &'a str,

    /// The description of the policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,

    /// The language of the policy (e.g., "expr").
    pub language: &'a str,

    /// The parameters of the policy.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: &'a Vec<PolicyParameter>,

    /// The policy expression.
    pub expression: &'a str,

    /// Optional metadata associated with the policy.
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: &'a std::collections::HashMap<String, serde_json::Value>,
}

/// An error returned from [`CreatePolicy`].
#[derive(Debug, Error)]
pub enum CreatePolicyError {}

impl From<CreatePolicyError> for WorkOsError<CreatePolicyError> {
    fn from(err: CreatePolicyError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Create a policy](https://workos.com/docs/reference/fga/policy/create)
#[async_trait]
pub trait CreatePolicy {
    /// Creates a new policy in the current environment.
    ///
    /// [WorkOS Docs: Create a policy](https://workos.com/docs/reference/fga/policy/create)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    /// use std::collections::HashMap;
    ///
    /// # async fn run() -> WorkOsResult<(), CreatePolicyError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let parameters = vec![
    ///     PolicyParameter {
    ///         name: "clientIp".into(),
    ///         r#type: "string".into(),
    ///     },
    /// ];
    ///
    /// let policy = workos
    ///     .fga()
    ///     .create_policy(&CreatePolicyParams {
    ///         name: "ip_equal",
    ///         description: Some("Check if the client IP is in the range 192.168.x.x"),
    ///         language: "expr",
    ///         parameters: &parameters,
    ///         expression: "clientIp matches \"192\\.168\\..*\\..*\"",
    ///         metadata: &HashMap::new(),
    ///     })
    ///     .await?;
    ///
    /// println!("Created policy: {:?}", policy);
    /// # Ok(())
    /// # }
    /// ```
    async fn create_policy(
        &self,
        params: &CreatePolicyParams<'_>,
    ) -> WorkOsResult<Policy, CreatePolicyError>;
}

#[async_trait]
impl CreatePolicy for Fga<'_> {
    async fn create_policy(
        &self,
        params: &CreatePolicyParams<'_>,
    ) -> WorkOsResult<Policy, CreatePolicyError> {
        let url = self.workos.base_url().join("/fga/v1/policies")?;
        let policy = self
            .workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&params)
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
    use crate::fga::PolicyParameter;

    #[tokio::test]
    async fn it_calls_the_create_policy_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/policies")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(201)
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
                    "expression": "clientIp matches \"192\\.168\\..*\\..*\"",
                    "metadata": {}
                })
                .to_string(),
            )
            .create_async()
            .await;

        let parameters = vec![
            PolicyParameter {
                name: "clientIp".into(),
                r#type: "string".into(),
            },
        ];

        let policy = workos
            .fga()
            .create_policy(&CreatePolicyParams {
                name: "ip_equal",
                description: Some("Check if the client IP is in the range 192.168.x.x"),
                language: "expr",
                parameters: &parameters,
                expression: "clientIp matches \"192\\.168\\..*\\..*\"",
                metadata: &std::collections::HashMap::new(),
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