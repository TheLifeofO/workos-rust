use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::fga::{Warrant, Fga, Subject};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// Parameters for [`CreateWarrant`].
#[derive(Debug, Serialize)]
pub struct CreateWarrantParams<'a> {
    /// The type of the resource.
    pub resource_type: &'a str,

    /// The unique identifier of the resource.
    pub resource_id: &'a str,

    /// The relation to grant.
    pub relation: &'a str,

    /// The subject to grant the relation to.
    pub subject: Subject,

    /// A boolean expression that must evaluate to true for this warrant to apply
    pub policy: Option<String>,
}

/// An error returned from [`CreateWarrant`].
#[derive(Debug, Error)]
pub enum CreateWarrantError {}

impl From<CreateWarrantError> for WorkOsError<CreateWarrantError> {
    fn from(err: CreateWarrantError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Create a warrant](https://workos.com/docs/reference/fga/warrant/create)
#[async_trait]
pub trait CreateWarrant {
    /// Creates a new warrant in the current environment.
    ///
    /// [WorkOS Docs: Create a warrant](https://workos.com/docs/reference/fga/warrant/create)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::fga::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), CreateWarrantError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let warrant = workos
    ///     .fga()
    ///     .create_warrant(&CreateWarrantParams {
    ///         resource_type: "document",
    ///         resource_id: "doc_123",
    ///         relation: "viewer",
    ///         subject: Subject {
    ///             resource_type: String::from("user"),
    ///             resource_id: String::from("user_123"),
    ///         },
    ///        policy: None,
    ///     })
    ///     .await?;
    ///
    /// println!("Created warrant: {:?}", warrant);
    /// # Ok(())
    /// # }
    /// ```
    async fn create_warrant(
        &self,
        params: &CreateWarrantParams<'_>,
    ) -> WorkOsResult<Warrant, CreateWarrantError>;
}

#[async_trait]
impl CreateWarrant for Fga<'_> {
    async fn create_warrant(
        &self,
        params: &CreateWarrantParams<'_>,
    ) -> WorkOsResult<Warrant, CreateWarrantError> {
        let url = self.workos.base_url().join("/fga/v1/warrants")?;
        let warrant = self
            .workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&params)
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Warrant>()
            .await?;

        Ok(warrant)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};
    use crate::fga::Subject;

    #[tokio::test]
    async fn it_calls_the_create_warrant_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("POST", "/fga/v1/warrants")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(201)
            .with_body(
                json!({
                    "resource_type": "document",
                    "resource_id": "doc_123",
                    "relation": "viewer",
                    "subject": {
                        "resource_type": "user",
                        "resource_id": "user_123"
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let warrant = workos
            .fga()
            .create_warrant(&CreateWarrantParams {
                resource_type: "document",
                resource_id: "doc_123",
                relation: "viewer",
                subject: Subject {
                    resource_type: String::from("user"),
                    resource_id: String::from("user_123"),
                },
                policy: None,
            })
            .await
            .unwrap();

        assert_eq!(warrant.resource_type, "document");
        assert_eq!(warrant.resource_id, "doc_123");
        assert_eq!(warrant.relation, "viewer");
        assert_eq!(warrant.subject.resource_type, "user");
        assert_eq!(warrant.subject.resource_id, "user_123");
    }
}