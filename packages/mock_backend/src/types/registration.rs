use async_graphql::{ID, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

/// Standard response envelope used by all mutations
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct DefaultResponse {
    pub status: String,
    #[graphql(name = "RequestId")]
    pub request_id: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    #[graphql(name = "ResponseMessage")]
    pub response_message: String,
}

impl DefaultResponse {
    pub fn success(code: &str, message: &str) -> Self {
        Self {
            status: "success".to_string(),
            request_id: format!("mock-req-{}", chrono::Utc::now().timestamp_millis()),
            response_code: code.to_string(),
            response_message: message.to_string(),
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            status: "error".to_string(),
            request_id: format!("mock-req-{}", chrono::Utc::now().timestamp_millis()),
            response_code: code.to_string(),
            response_message: message.to_string(),
        }
    }
}

/// User info returned in referral verification
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ReferralUser {
    pub uid: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
}

/// Response for verifyReferralString mutation
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ReferralResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<ReferralUser>>,
    pub meta: DefaultResponse,
}

/// Input for register mutation
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct RegistrationInput {
    pub email: String,
    pub password: String,
    pub username: String,
    pub pkey: Option<String>,
    #[graphql(name = "referralUuid")]
    pub referral_uuid: Option<ID>,
}

/// Response for register mutation
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    pub userid: Option<String>,
    pub meta: DefaultResponse,
}

/// Response for verifyAccount mutation
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct VerifyAccountResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    pub meta: DefaultResponse,
}
