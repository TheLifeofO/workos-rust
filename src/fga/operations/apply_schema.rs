use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::Fga;
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`ApplySchema`].
#[derive(Debug, Serialize)]
pub struct ApplySchemaParams<'a> {
    /// The schema to apply.
    pub schema: &'a str,
}

/// An error returned from [`ApplySchema`].
#[derive(Debug, Error)]
#[derive(PartialEq)]
pub enum ApplySchemaError {}

impl From<ApplySchemaError> for WorkOsError<ApplySchemaError> {
    fn from(err: ApplySchemaError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Apply Schema](https://workos.com/docs/reference/fga/schema/apply)
#[async_trait]
pub trait ApplySchema {
    /// Sets resource types and policies in the current environment.
    ///
    /// This endpoint performs a batch operation which will override your entire schema for the environment.
    /// Any existing resource types and policies not included in the request will be deleted.
    ///
    /// [WorkOS Docs: Apply Schema](https://workos.com/docs/reference/fga/schema/apply)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), ApplySchemaError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let schema = r#"{
    ///     "resource_types": [
    ///         {
    ///             "type": "document",
    ///             "relations": {
    ///                 "owner": { "this": {} },
    ///                 "viewer": { "this": {} }
    ///             }
    ///         }
    ///     ],
    ///     "policies": [
    ///         {
    ///             "name": "example_policy",
    ///             "description": "Example policy",
    ///             "language": "expr",
    ///             "parameters": [],
    ///             "expression": "true"
    ///         }
    ///     ]
    /// }"#;
    ///
    /// workos
    ///     .fga()
    ///     .apply_schema(&ApplySchemaParams { schema })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn apply_schema(
        &self,
        params: &ApplySchemaParams<'_>,
    ) -> WorkOsResult<(), ApplySchemaError>;
}

#[async_trait]
impl ApplySchema for Fga<'_> {
    async fn apply_schema(
        &self,
        params: &ApplySchemaParams<'_>,
    ) -> WorkOsResult<(), ApplySchemaError> {
        let url = self.workos.base_url().join("/fga/v1/schema")?;
        self.workos
            .client()
            .put(url)
            .bearer_auth(self.workos.key())
            .body(params.schema.to_string())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_apply_schema_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("PUT", "/fga/v1/schema")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(204)
            .create_async()
            .await;

        let schema = r#"{
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
                    "parameters": [],
                    "expression": "true"
                }
            ]
        }"#;

        let result = workos
            .fga()
            .apply_schema(&ApplySchemaParams { schema })
            .await;

        assert_eq!(result.is_ok(), true);
    }
}