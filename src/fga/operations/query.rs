use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{Fga, QueryResponse};
use crate::{PaginatedList, PaginationParams, ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`Query`].
#[derive(Debug, Serialize)]
pub struct QueryParams<'a> {
    /// A valid token string from a previous write operation or latest
    pub warrant_token: Option<&'a str>,

    /// A query written in the Query Language.
    pub q: &'a str,

    /// A serialized, url-safe JSON object containing contextual data to use while resolving the query.
    pub context: Option<&'a str>,

    /// The pagination parameters to use when listing policies.
    #[serde(flatten)]
    pub pagination: PaginationParams<'a>,
}

/// An error returned from [`Query`].
#[derive(Debug, Error)]
pub enum QueryError {}

impl From<QueryError> for WorkOsError<QueryError> {
    fn from(err: QueryError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Query](https://workos.com/docs/reference/fga/query)
#[async_trait]
pub trait Query {
    /// Executes a query to list the set of subjects that have access to a particular resource or to list the set of resources a particular subject has access to.
    ///
    /// [WorkOS Docs: Query](https://workos.com/docs/reference/fga/query)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::{PaginationParams, WorkOsResult};
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), QueryError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let result = workos
    ///     .fga()
    ///     .query(None, &QueryParams {
    ///        warrant_token: None,
    ///        q: "document:doc_123 viewer",
    ///        context: None,
    ///        pagination: PaginationParams::default(),
    ///     })
    ///     .await?;
    ///
    /// println!("Query result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    async fn query(&self, token: Option<String>,params: &QueryParams<'_>) -> WorkOsResult<PaginatedList<QueryResponse>, QueryError>;
}

#[async_trait]
impl Query for Fga<'_> {
    async fn query(&self, token: Option<String>, params: &QueryParams<'_>) -> WorkOsResult<PaginatedList<QueryResponse>, QueryError> {
        let url = self.workos.base_url().join("/fga/v1/query")?;
        let result = self
            .workos
            .client()
            .get(url)
            .bearer_auth(if let Some(token) = token {
                token
            } else {
                self.workos.key().to_string()
            })
            .json(&params)
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<PaginatedList<QueryResponse>>()
            .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_query_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/query")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "resource": "document:doc_123",
                    "relation": "viewer",
                    "subjects": ["user_123", "user_456"]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let result = workos
            .fga()
            .query(None, &QueryParams {
                warrant_token: None,
                q: "",
                context: None,
                pagination: Default::default(),
            })
            .await
            .unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].resource.resource_id, "document:doc_123");
    }
}
