use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;

use crate::fga::{ResourceType, Fga};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`GetResourceType`].
#[derive(Debug)]
pub struct GetResourceTypeParams<'a> {
    /// The name of the resource type to retrieve.
    pub resource_type: &'a str,
}

/// An error returned from [`GetResourceType`].
#[derive(Debug, Error)]
pub enum GetResourceTypeError {}

impl From<GetResourceTypeError> for WorkOsError<GetResourceTypeError> {
    fn from(err: GetResourceTypeError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Get a resource type](https://workos.com/docs/reference/fga/resource-type/get)
#[async_trait]
pub trait GetResourceType {
    /// Retrieves the definition of an existing resource type.
    ///
    /// [WorkOS Docs: Get a resource type](https://workos.com/docs/reference/fga/resource-type/get)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), GetResourceTypeError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let resource_type = workos
    ///     .fga()
    ///     .get_resource_type(&GetResourceTypeParams {
    ///         resource_type: "document",
    ///     })
    ///     .await?;
    ///
    /// println!("Resource type: {:?}", resource_type);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_resource_type(
        &self,
        params: &GetResourceTypeParams<'_>,
    ) -> WorkOsResult<ResourceType, GetResourceTypeError>;
}

#[async_trait]
impl GetResourceType for Fga<'_> {
    async fn get_resource_type(
        &self,
        params: &GetResourceTypeParams<'_>,
    ) -> WorkOsResult<ResourceType, GetResourceTypeError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/fga/v1/resource-types/{}", params.resource_type))?;

        let resource_type = self
            .workos
            .client()
            .get(url)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<ResourceType>()
            .await?;

        Ok(resource_type)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use crate::{ApiKey, WorkOs};

    use super::*;

    #[tokio::test]
    async fn it_calls_the_get_resource_type_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/resource-types/document")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "type": "document",
                    "relations": {
                        "owner": {
                            "inherit": {
                                "relation": "owner",
                                "from": "parent"
                            }
                        },
                        "viewer": {
                            "union": [
                                { "this": {} },
                                { "inherit": { "relation": "viewer", "from": "parent" } }
                            ]
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let resource_type = workos
            .fga()
            .get_resource_type(&GetResourceTypeParams {
                resource_type: "document",
            })
            .await
            .unwrap();

        assert_eq!(resource_type.r#type, "document");
    }
}