use serde::{Deserialize, Serialize};

use crate::{KnownOrUnknown};
use crate::user_management::UserId;

/// The type of verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationType {
    /// Email verification.
    EmailVerification,
}

/// The status of a verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// The verification succeeded.
    Succeeded,

    /// The verification failed.
    Failed,

    /// The verification is pending.
    Pending,

    /// The verification was cancelled.
    Cancelled,

    /// The verification expired.
    Expired,
}

/// A verification event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verification {
    /// The type of verification.
    pub r#type: KnownOrUnknown<VerificationType, String>,

    /// The status of the verification.
    pub status: KnownOrUnknown<VerificationStatus, String>,

    /// The ID of the user being verified.
    pub user_id: UserId,

    /// The email address being verified (if applicable).
    pub email: Option<String>,

    /// The IP address from which the verification was initiated.
    pub ip_address: String,

    /// The user agent string of the client that initiated the verification.
    pub user_agent: String,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::{KnownOrUnknown};
    use crate::user_management::UserId;
    use super::{Verification, VerificationStatus, VerificationType};

    #[test]
    fn it_deserializes_email_verification() {
        let verification: Verification = serde_json::from_str(
            &json!({
                "type": "email_verification",
                "status": "succeeded",
                "user_id": "user_01E4ZCR3C5A4QZ2Z2JQXGKZJ9E",
                "email": "todd@example.com",
                "ip_address": "192.0.2.1",
                "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.81 Safari/537.36",
            })
            .to_string(),
        )
        .unwrap();

        assert_eq!(
            verification,
            Verification {
                r#type: KnownOrUnknown::Known(VerificationType::EmailVerification),
                status: KnownOrUnknown::Known(VerificationStatus::Succeeded),
                user_id: UserId::from("user_01E4ZCR3C5A4QZ2Z2JQXGKZJ9E"),
                email: Some("todd@example.com".to_string()),
                ip_address: "192.0.2.1".to_string(),
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.81 Safari/537.36".to_string(),
            }
        );
    }

    #[test]
    fn it_deserializes_unknown_verification_types() {
        let verification: Verification = serde_json::from_str(
            &json!({
                "type": "unknown_verification_type",
                "status": "succeeded",
                "user_id": "user_01E4ZCR3C5A4QZ2Z2JQXGKZJ9E",
                "email": "todd@example.com",
                "phone_number": null,
                "ip_address": "192.0.2.1",
                "user_agent": "Mozilla/5.0",
                "created_at": "2021-06-25T19:07:33.155Z",
                "updated_at": "2021-06-25T19:07:33.155Z"
            })
            .to_string(),
        )
        .unwrap();

        assert_eq!(
            verification.r#type,
            KnownOrUnknown::Unknown("unknown_verification_type".to_string())
        );
    }

    #[test]
    fn it_deserializes_unknown_verification_status() {
        let verification: Verification = serde_json::from_str(
            &json!({
                "type": "email_verification",
                "status": "unknown_status",
                "user_id": "user_01E4ZCR3C5A4QZ2Z2JQXGKZJ9E",
                "email": "todd@example.com",
                "phone_number": null,
                "ip_address": "192.0.2.1",
                "user_agent": "Mozilla/5.0",
                "created_at": "2021-06-25T19:07:33.155Z",
                "updated_at": "2021-06-25T19:07:33.155Z"
            })
            .to_string(),
        )
        .unwrap();

        assert_eq!(
            verification.status,
            KnownOrUnknown::Unknown("unknown_status".to_string())
        );
    }
}
