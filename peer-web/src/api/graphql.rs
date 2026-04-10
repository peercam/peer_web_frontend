//! GraphQL client for communicating with the Peer backend.
//!
//! This module provides a type-safe interface for sending GraphQL
//! queries and mutations to the backend.

use crate::models::ApiError;
use serde::{de::DeserializeOwned, Serialize};

/// GraphQL request body.
#[derive(Debug, Serialize)]
pub struct GraphQLRequest<V: Serialize> {
    pub query: &'static str,
    pub variables: V,
}

/// GraphQL response envelope.
#[derive(Debug, serde::Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<crate::models::GraphQLError>>,
}

/// Placeholder for the GraphQL endpoint URL.
/// In production, this will be read from environment variables.
pub fn get_graphql_endpoint() -> String {
    std::env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string())
}

/// Send a GraphQL mutation to the backend.
///
/// This is a placeholder signature — full implementation in Step 3.
///
/// # Type Parameters
/// - `V`: The variables type (must be serializable)
/// - `T`: The expected response data type (must be deserializable)
///
/// # Arguments
/// - `query`: The GraphQL mutation string
/// - `variables`: The mutation variables
/// - `auth_token`: Optional bearer token for authenticated requests
///
/// # Returns
/// The deserialized response data or an error.
#[cfg(feature = "ssr")]
pub async fn mutate<V, T>(
    _query: &'static str,
    _variables: V,
    _auth_token: Option<&str>,
) -> Result<T, ApiError>
where
    V: Serialize + Send,
    T: DeserializeOwned,
{
    // Placeholder — implementation in Step 3
    Err(ApiError::Unexpected(
        "GraphQL client not yet implemented".to_string(),
    ))
}

/// Marker module for GraphQL query/mutation strings.
pub mod queries {
    /// Referral verification mutation.
    pub const VERIFY_REFERRAL: &str = r#"
        mutation VerifyReferralString($referralString: String!) {
            verifyReferralString(referralString: $referralString) {
                ResponseCode
                affectedRows {
                    uid
                    username
                    slug
                    img
                }
                status
            }
        }
    "#;

    /// User registration mutation.
    pub const REGISTER: &str = r#"
        mutation Register($input: RegistrationInput!) {
            register(input: $input) {
                status
                ResponseCode
                userid
            }
        }
    "#;

    /// Account verification mutation.
    pub const VERIFY_ACCOUNT: &str = r#"
        mutation VerifyAccount($userId: ID!) {
            verifyAccount(userid: $userId) {
                status
                ResponseCode
            }
        }
    "#;
}
