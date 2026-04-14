use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

/// Response for login and refreshToken mutations.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AuthPayload {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "accessToken")]
    pub access_token: Option<String>,
    #[graphql(name = "refreshToken")]
    pub refresh_token: Option<String>,
}

impl AuthPayload {
    pub fn success(code: &str, access: &str, refresh: &str) -> Self {
        Self {
            status: "success".to_string(),
            response_code: Some(code.to_string()),
            access_token: Some(access.to_string()),
            refresh_token: Some(refresh.to_string()),
        }
    }

    pub fn error(code: &str) -> Self {
        Self {
            status: "error".to_string(),
            response_code: Some(code.to_string()),
            access_token: None,
            refresh_token: None,
        }
    }
}

/// Response for logout mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct LogoutPayload {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
}

/// Response for requestPasswordReset mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ResetPasswordRequestResponse {
    pub meta: DefaultResponse,
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "nextAttemptAt")]
    pub next_attempt_at: Option<String>,
}

/// Response for contactus mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ContactusResponse {
    pub meta: DefaultResponse,
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ContactusResponsePayload>,
}

/// Payload within ContactusResponse.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ContactusResponsePayload {
    pub msgid: String,
    pub email: String,
    pub name: String,
    pub message: String,
    pub ip: String,
    pub createdat: String,
}
