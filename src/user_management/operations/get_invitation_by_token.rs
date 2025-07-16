use async_trait::async_trait;
use thiserror::Error;

use crate::user_management::{Invitation, InvitationToken, UserManagement};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// An error returned from [`GetInvitationByToken`].
#[derive(Debug, Error)]
pub enum GetInvitationByTokenError {}

impl From<GetInvitationByTokenError> for WorkOsError<GetInvitationByTokenError> {
    fn from(err: GetInvitationByTokenError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Find an invitation by token](https://workos.com/docs/reference/user-management/invitation/find-by-token)
#[async_trait]
pub trait GetInvitationByToken {
    /// Retrieve an existing invitation using the token.
    ///
    /// [WorkOS Docs: Find an invitation by token](https://workos.com/docs/reference/user-management/invitation/find-by-token)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::user_management::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    /// #
    /// use workos_sdk::organizations::OrganizationId;
    ///
    /// async fn run() -> WorkOsResult<(), GetInvitationByTokenError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let invitation = workos
    ///     .user_management()
    ///     .get_invitation_by_token(&InvitationToken::from("Z1uX3RbwcIl5fIGJJJCXXisdI"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn get_invitation_by_token(
        &self,
        token: &InvitationToken,
    ) -> WorkOsResult<Invitation, GetInvitationByTokenError>;
}

#[async_trait]
impl GetInvitationByToken for UserManagement<'_> {
    async fn get_invitation_by_token(
        &self,
        token: &InvitationToken,
    ) -> WorkOsResult<Invitation, GetInvitationByTokenError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("user_management/invitations/by_token/{token}"))?;

        let invitation = self
            .workos
            .client()
            .get(url)
            .bearer_auth(self.workos.key())
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
    use crate::user_management::InvitationId;
    use super::*;

    #[tokio::test]
    async fn it_calls_the_get_invitation_by_token_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock(
                "GET",
                "/user_management/invitations/by_token/Z1uX3RbwcIl5fIGJJJCXXisdI",
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
            .get_invitation_by_token(&InvitationToken::from("Z1uX3RbwcIl5fIGJJJCXXisdI"))
            .await
            .unwrap();

        assert_eq!(invitation.id, InvitationId::from("invitation_01E4ZCR3C56J083X43JQXF3JK5"));
    }
}
