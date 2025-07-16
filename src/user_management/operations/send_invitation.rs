use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::organizations::OrganizationId;
use crate::user_management::{Invitation, UserId, UserManagement};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// The parameters for [`SendInvitation`].
#[derive(Debug, Serialize)]
pub struct SendInvitationParams<'a> {
    /// The email address of the recipient.
    pub email: &'a str,

    /// The ID of the organization that the recipient will join.
    pub organization_id: Option<&'a OrganizationId>,

    /// How many days the invitations will be valid for.
    /// Must be between 1 and 30 days. Defaults to 7 days if not specified.
    pub expires_in_days: Option<&'a usize>,

    /// The ID of the user who invites the recipient. The invitation email will mention the name of this user.
    pub inviter_user_id: Option<&'a UserId>,

    /// The role that the recipient will receive when they join the organization in the invitation.
    pub role_slug: Option<&'a str>,
}

/// An error returned from [`SendInvitation`].
#[derive(Debug, Error)]
pub enum SendInvitationError {}

impl From<SendInvitationError> for WorkOsError<SendInvitationError> {
    fn from(err: SendInvitationError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Send an invitation](https://workos.com/docs/reference/user-management/invitation/send)
#[async_trait]
pub trait SendInvitation {
    /// Sends an invitation email to the recipient.
    ///
    /// [WorkOS Docs: Send an invitation](https://workos.com/docs/reference/user-management/invitation/send)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::user_management::*;
    /// use workos_sdk::{ApiKey, WorkOs};    ///
    /// #
    /// use workos_sdk::organizations::OrganizationId;
    ///
    /// async fn run() -> WorkOsResult<(), SendInvitationError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let invitation = workos
    ///     .user_management()
    ///     .send_invitation(&SendInvitationParams {
    ///          email: "marcelina.davis@example.com",
    ///          organization_id: Some(&OrganizationId::from("org_01E4ZCR3C56J083X43JQXF3JK5")),
    ///          expires_in_days: Some(&7),
    ///          inviter_user_id: Some(&UserId::from("user_01HYGBX8ZGD19949T3BM4FW1C3")),
    ///          role_slug: Some("member"),
    ///     })
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn send_invitation(
        &self,
        params: &SendInvitationParams<'_>,
    ) -> WorkOsResult<Invitation, SendInvitationError>;
}

#[async_trait]
impl SendInvitation for UserManagement<'_> {
    async fn send_invitation(
        &self,
        params: &SendInvitationParams<'_>,
    ) -> WorkOsResult<Invitation, SendInvitationError> {
        let url = self
            .workos
            .base_url()
            .join("/user_management/invitations")?;

        let invitation = self
            .workos
            .client()
            .post(url)
            .bearer_auth(self.workos.key())
            .json(&params)
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Invitation>()
            .await?;

        Ok(invitation)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use crate::{ApiKey, WorkOs};

    use super::*;

    #[tokio::test]
    async fn it_calls_the_send_invitation_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock(
                "POST",
                "/user_management/invitations",
            )
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                  "object": "invitation",
                  "id": "invitation_01E4ZCR3C56J083X43JQXF3JK5",
                  "email": "marcelina.davis@example.com",
                  "state": "pending",
                  "accepted_at": null,
                  "revoked_at": null,
                  "expires_at": "2021-07-01T19:07:33.155Z",
                  "token": "Z1uX3RbwcIl5fIGJJJCXXisdI",
                  "accept_invitation_url": "https://your-app.com/invite?invitation_token=Z1uX3RbwcIl5fIGJJJCXXisdI",
                  "organization_id": "org_01E4ZCR3C56J083X43JQXF3JK5",
                  "inviter_user_id": "user_01HYGBX8ZGD19949T3BM4FW1C3",
                  "created_at": "2021-06-25T19:07:33.155Z",
                  "updated_at": "2021-06-25T19:07:33.155Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let invitation = workos
            .user_management()
            .send_invitation(&SendInvitationParams {
                email: "marcelina.davis@example.com",
                organization_id: Some(&OrganizationId::from("org_01E4ZCR3C56J083X43JQXF3JK5")),
                expires_in_days: Some(&7),
                inviter_user_id: Some(&UserId::from("user_01HYGBX8ZGD19949T3BM4FW1C3")),
                role_slug: Some("member"),
            })
            .await
            .unwrap();

        assert_eq!(invitation.email, String::from("marcelina.davis@example.com"))
    }
}
