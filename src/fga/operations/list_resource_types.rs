use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{ResourceType, Fga};
use crate::{PaginatedList, PaginationParams, ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`ListResourceTypes`].
#[derive(Debug, Default, Serialize)]
pub struct ListResourceTypesParams<'a> {
    /// Pagination parameters.
    #[serde(flatten)]
    pub pagination: PaginationParams<'a>,
}

/// An error returned from [`ListResourceTypes`].
#[derive(Debug, Error)]
pub enum ListResourceTypesError {}

impl From<ListResourceTypesError> for WorkOsError<ListResourceTypesError> {
    fn from(err: ListResourceTypesError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: List resource types](https://workos.com/docs/reference/fga/resource-type/list)
#[async_trait]
pub trait ListResourceTypes {
    /// Retrieves a paginated list of all resource-type definitions.
    ///
    /// [WorkOS Docs: List resource types](https://workos.com/docs/reference/fga/resource-type/list)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), ListResourceTypesError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let paginated = workos
    ///     .fga()
    ///     .list_resource_types(&ListResourceTypesParams::default())
    ///     .await?;
    ///
    /// for rt in paginated.data {
    ///     println!("resource type: {}", rt.r#type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn list_resource_types(
        &self,
        params: &ListResourceTypesParams<'_>,
    ) -> WorkOsResult<PaginatedList<ResourceType>, ListResourceTypesError>;
}

#[async_trait]
impl ListResourceTypes for Fga<'_> {
    async fn list_resource_types(
        &self,
        params: &ListResourceTypesParams<'_>,
    ) -> WorkOsResult<PaginatedList<ResourceType>, ListResourceTypesError> {
        let url = self.workos.base_url().join("/fga/v1/resource-types")?;
        let list = self
            .workos
            .client()
            .get(url)
            .query(&params)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<PaginatedList<ResourceType>>()
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
    async fn it_calls_the_list_resource_types_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/resource-types")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "data": [
                        {
                            "type": "document",
                            "relations": {
                                "owner": { "this": {} },
                                "viewer": { "this": {} }
                            }
                        },
                        {
                            "type": "folder",
                            "relations": {
                                "owner": { "this": {} }
                            }
                        }
                    ],
                    "list_metadata": {
                        "before": null,
                        "after": null
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let result = workos
            .fga()
            .list_resource_types(&ListResourceTypesParams::default())
            .await
            .unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].r#type, "document");
    }
}