use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{Resource, Fga};
use crate::{PaginatedList, PaginationParams, ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`ListResources`].
#[derive(Debug, Default, Serialize)]
pub struct ListResourcesParams<'a> {
    /// Pagination controls.
    #[serde(flatten)]
    pub pagination: PaginationParams<'a>,

    /// Filter by resource type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<&'a str>,
}

/// An error returned from [`ListResources`].
#[derive(Debug, Error)]
pub enum ListResourcesError {}

impl From<ListResourcesError> for WorkOsError<ListResourcesError> {
    fn from(err: ListResourcesError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: List resources](https://workos.com/docs/reference/fga/resource/list)
#[async_trait]
pub trait ListResources {
    /// Retrieves a paginated list of resources matching the filters.
    ///
    /// [WorkOS Docs: List resources](https://workos.com/docs/reference/fga/resource/list)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), ListResourcesError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let resources = workos
    ///     .fga()
    ///     .list_resources(&ListResourcesParams {
    ///         resource_type: Some("document"),
    ///         ..Default::default()
    ///     })
    ///     .await?;
    ///
    /// println!("Found {} resources", resources.data.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn list_resources(
        &self,
        params: &ListResourcesParams<'_>,
    ) -> WorkOsResult<PaginatedList<Resource>, ListResourcesError>;
}

#[async_trait]
impl ListResources for Fga<'_> {
    async fn list_resources(
        &self,
        params: &ListResourcesParams<'_>,
    ) -> WorkOsResult<PaginatedList<Resource>, ListResourcesError> {
        let url = self.workos.base_url().join("/fga/v1/resources")?;
        let list = self
            .workos
            .client()
            .get(url)
            .query(&params)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<PaginatedList<Resource>>()
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
    async fn it_calls_the_list_resources_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/resources")
            .match_query("resource_type=document")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "data": [
                        {
                            "type": "document",
                            "id": "doc_abc",
                            "metadata": {}
                        },
                        {
                            "type": "document",
                            "id": "doc_def",
                            "metadata": {}
                        }
                    ],
                    "list_metadata": { "before": null, "after": null }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let result = workos
            .fga()
            .list_resources(&ListResourcesParams {
                resource_type: Some("document"),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].resource_type, "document");
    }
}