use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{Warrant, Fga};
use crate::{PaginatedList, PaginationParams, ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`ListWarrants`].
#[derive(Debug, Default, Serialize)]
pub struct ListWarrantsParams<'a> {
    /// Pagination controls.
    #[serde(flatten)]
    pub pagination: PaginationParams<'a>,

    /// Filter by subject type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_type: Option<&'a str>,

    /// Filter by subject id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<&'a str>,

    /// Filter by relation name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<&'a str>,

    /// Filter by resource type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<&'a str>,

    /// Filter by resource id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<&'a str>,
}

/// An error returned from [`ListWarrants`].
#[derive(Debug, Error)]
pub enum ListWarrantsError {}

impl From<ListWarrantsError> for WorkOsError<ListWarrantsError> {
    fn from(err: ListWarrantsError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: List Warrants](https://workos.com/docs/reference/fga/warrant/list)
#[async_trait]
pub trait ListWarrants {
    /// Retrieves a paginated list of warrants matching the filters.
    ///
    /// [WorkOS Docs: List Warrants](https://workos.com/docs/reference/fga/warrant/list)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), ListWarrantsError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let warrants = workos
    ///     .fga()
    ///     .list_warrants(&ListWarrantsParams {
    ///         resource_type: Some("document"),
    ///         relation: Some("viewer"),
    ///         ..Default::default()
    ///     })
    ///     .await?;
    ///
    /// println!("Found {} warrants", warrants.data.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn list_warrants(
        &self,
        params: &ListWarrantsParams<'_>,
    ) -> WorkOsResult<PaginatedList<Warrant>, ListWarrantsError>;
}

#[async_trait]
impl ListWarrants for Fga<'_> {
    async fn list_warrants(
        &self,
        params: &ListWarrantsParams<'_>,
    ) -> WorkOsResult<PaginatedList<Warrant>, ListWarrantsError> {
        let url = self.workos.base_url().join("/fga/v1/warrants")?;
        let list = self
            .workos
            .client()
            .get(url)
            .query(&params)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<PaginatedList<Warrant>>()
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
    async fn it_calls_the_list_warrants_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/warrants")
            .match_query("resource_type=document&relation=viewer")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "data": [
                        {
                            "resource_type": "document",
                            "resource_id": "doc_abc",
                            "relation": "viewer",
                            "subject": {
                                "resource_type": "user",
                                "resource_id": "user_123"
                            }
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
            .list_warrants(&ListWarrantsParams {
                resource_type: Some("document"),
                relation: Some("viewer"),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].relation, "viewer");
    }
}