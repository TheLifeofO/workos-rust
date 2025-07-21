use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{Policy, Fga};
use crate::{PaginatedList, PaginationParams, ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`ListPolicies`].
#[derive(Debug, Default, Serialize)]
pub struct ListPoliciesParams<'a> {
    /// Pagination controls.
    #[serde(flatten)]
    pub pagination: PaginationParams<'a>,
}

/// An error returned from [`ListPolicies`].
#[derive(Debug, Error)]
pub enum ListPoliciesError {}

impl From<ListPoliciesError> for WorkOsError<ListPoliciesError> {
    fn from(err: ListPoliciesError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: List policies](https://workos.com/docs/reference/fga/policy/list)
#[async_trait]
pub trait ListPolicies {
    /// Retrieves a paginated list of all policies.
    ///
    /// [WorkOS Docs: List policies](https://workos.com/docs/reference/fga/policy/list)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), ListPoliciesError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let policies = workos
    ///     .fga()
    ///     .list_policies(&ListPoliciesParams::default())
    ///     .await?;
    ///
    /// println!("Found {} policies", policies.data.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn list_policies(
        &self,
        params: &ListPoliciesParams<'_>,
    ) -> WorkOsResult<PaginatedList<Policy>, ListPoliciesError>;
}

#[async_trait]
impl ListPolicies for Fga<'_> {
    async fn list_policies(
        &self,
        params: &ListPoliciesParams<'_>,
    ) -> WorkOsResult<PaginatedList<Policy>, ListPoliciesError> {
        let url = self.workos.base_url().join("/fga/v1/policies")?;
        let list = self
            .workos
            .client()
            .get(url)
            .query(&params)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<PaginatedList<Policy>>()
            .await?;

        Ok(list)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_list_policies_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/policies")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "data": [
                        {
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
                        }
                    ],
                    "list_metadata": { "before": null, "after": null }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let policies = workos
            .fga()
            .list_policies(&ListPoliciesParams::default())
            .await
            .unwrap();

        assert_eq!(policies.data.len(), 1);
        assert_eq!(policies.data[0].name, "ip_equal");
        assert_eq!(policies.data[0].description.clone().unwrap(), "Check if the client IP is in the range 192.168.x.x");
        assert_eq!(policies.data[0].language, "expr");
        assert_eq!(policies.data[0].parameters.len(), 1);
        assert_eq!(policies.data[0].parameters[0].name, "clientIp");
        assert_eq!(policies.data[0].parameters[0].r#type, "string");
        assert_eq!(policies.data[0].expression, "clientIp matches \"192\\.168\\..*\\..*\"");
    }
}