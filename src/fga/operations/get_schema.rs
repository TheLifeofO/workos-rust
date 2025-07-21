use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fga::{Fga, Policy, ResourceType};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Represents the authorization model in your FGA environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    /// The resource type definitions.
    pub resource_types: Vec<ResourceType>,

    /// The policies.
    pub policies: Vec<Policy>,
}

/// Parameters for [`GetSchema`].
#[derive(Debug, Serialize)]
pub struct GetSchemaParams {}

/// An error returned from [`GetSchema`].
#[derive(Debug, Error)]
pub enum GetSchemaError {}

impl From<GetSchemaError> for WorkOsError<GetSchemaError> {
    fn from(err: GetSchemaError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Get a schema](https://workos.com/docs/reference/fga/schema/get)
#[async_trait]
pub trait GetSchema {
    /// Retrieves the authorization model for the current environment.
    ///
    /// [WorkOS Docs: Get a schema](https://workos.com/docs/reference/fga/schema/get)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), GetSchemaError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let schema = workos
    ///     .fga()
    ///     .get_schema(&GetSchemaParams {})
    ///     .await?;
    ///
    /// println!("Schema: {:?}", schema);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_schema(
        &self,
        _params: &GetSchemaParams,
    ) -> WorkOsResult<Schema, GetSchemaError>;
}

#[async_trait]
impl GetSchema for Fga<'_> {
    async fn get_schema(
        &self,
        _params: &GetSchemaParams,
    ) -> WorkOsResult<Schema, GetSchemaError> {
        let url = self.workos.base_url().join("/fga/v1/schema")?;
        let schema = self
            .workos
            .client()
            .get(url)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Schema>()
            .await?;

        Ok(schema)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_get_schema_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/fga/v1/schema")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "resource_types": [
                        {
                            "type": "document",
                            "relations": {
                                "owner": { "this": {} },
                                "viewer": { "this": {} }
                            }
                        }
                    ],
                    "policies": [
                        {
                            "name": "example_policy",
                            "description": "Example policy",
                            "language": "expr",
                            "parameters": [
                                {
                                    "name": "clientIp",
                                    "type": "string"
                                }
                            ],
                            "expression": "clientIp matches \"192\\.168\\..*\\..*\"",
                            "metadata": {}
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let schema = workos
            .fga()
            .get_schema(&GetSchemaParams {})
            .await
            .unwrap();

        assert_eq!(schema.resource_types.len(), 1);
        assert_eq!(schema.policies.len(), 1);
    }
}