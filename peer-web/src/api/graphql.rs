//! Generic GraphQL client for communicating with the Peer API.
//!
//! This module provides type-safe query and mutation helpers that:
//! - Serialize variables to JSON
//! - Send POST requests to the configured GraphQL endpoint
//! - Deserialize typed responses
//! - Handle the `{ data, errors }` envelope
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::api::graphql::mutate;
//! use crate::models::user::{ReferralVerifyResponse};
//!
//! let response: ReferralVerifyResponse = mutate(
//!     VERIFY_REFERRAL_MUTATION,
//!     serde_json::json!({ "referralString": "85d5f836-..." }),
//!     None, // No auth token for guest mutations
//! ).await?;
//! ```

use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use serde::de::DeserializeOwned;

use crate::models::common::{ApiError, GraphQLError};

/// GraphQL request envelope.
///
/// Matches the standard GraphQL-over-HTTP POST body format:
/// ```json
/// {
///   "query": "mutation { ... }",
///   "variables": { ... },
///   "operationName": "OptionalName"
/// }
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLRequest<V: Serialize> {
    pub query: &'static str,
    pub variables: V,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<&'static str>,
}

impl<V: Serialize> GraphQLRequest<V> {
    /// Create a new GraphQL request.
    pub fn new(query: &'static str, variables: V) -> Self {
        Self {
            query,
            variables,
            operation_name: None,
        }
    }

    /// Create a request with an explicit operation name.
    pub fn with_operation(query: &'static str, variables: V, operation_name: &'static str) -> Self {
        Self {
            query,
            variables,
            operation_name: Some(operation_name),
        }
    }
}

/// GraphQL response envelope.
///
/// All GraphQL responses follow this shape:
/// ```json
/// {
///   "data": { ... },      // Present on success (may be null)
///   "errors": [ ... ]     // Present on error (may be absent)
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphQLError>,
}

impl<T> GraphQLResponse<T> {
    /// Check if the response contains any GraphQL errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the first error message, if any.
    pub fn first_error_message(&self) -> Option<&str> {
        self.errors.first().map(|e| e.message.as_str())
    }

    /// Extract data, returning an error if data is missing or errors are present.
    pub fn into_result(self) -> Result<T, ApiError> {
        if let Some(err) = self.errors.first() {
            return Err(ApiError::GraphQL(err.message.clone()));
        }

        self.data.ok_or_else(|| {
            ApiError::Unexpected("GraphQL response contained neither data nor errors".to_string())
        })
    }
}

// ============================================================================
// Server-side implementation (SSR only)
// ============================================================================

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;
    use std::env;
    use std::time::Duration;

    /// Default timeout for GraphQL requests (10 seconds).
    const DEFAULT_TIMEOUT_SECS: u64 = 10;

    /// Get the GraphQL endpoint URL from environment.
    ///
    /// Reads `GRAPHQL_ENDPOINT` env var, falling back to local mock backend.
    pub fn get_endpoint() -> String {
        env::var("GRAPHQL_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string())
    }

    /// Build a configured reqwest client.
    fn build_client() -> Result<reqwest::Client, ApiError> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| ApiError::Network(format!("Failed to build HTTP client: {}", e)))
    }

    /// Execute a GraphQL mutation and deserialize the response.
    ///
    /// # Type Parameters
    ///
    /// - `V`: The variables type (must implement `Serialize`)
    /// - `R`: The expected response data type (must implement `DeserializeOwned`)
    ///
    /// # Arguments
    ///
    /// - `query`: The GraphQL mutation string
    /// - `variables`: Mutation variables (will be serialized to JSON)
    /// - `auth_token`: Optional Bearer token for authenticated mutations
    pub async fn mutate<V, R>(
        query: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        execute_request(query, variables, auth_token).await
    }

    /// Execute a GraphQL query and deserialize the response.
    ///
    /// Identical to `mutate` but semantically for queries.
    pub async fn query<V, R>(
        query_str: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        execute_request(query_str, variables, auth_token).await
    }

    /// Internal: Execute a GraphQL request (query or mutation).
    async fn execute_request<V, R>(
        query: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        let client = build_client()?;
        let endpoint = get_endpoint();

        let request_body = GraphQLRequest::new(query, variables);

        let mut request = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // Add authorization header if token provided
        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // Send the request
        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Network("Request timed out".to_string())
                } else if e.is_connect() {
                    ApiError::Network(format!("Failed to connect to {}: {}", endpoint, e))
                } else {
                    ApiError::Network(format!("Request failed: {}", e))
                }
            })?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Network(format!(
                "HTTP {} from {}: {}",
                status, endpoint, body
            )));
        }

        // Parse response body
        let response_text = response.text().await.map_err(|e| {
            ApiError::Deserialization(format!("Failed to read response body: {}", e))
        })?;

        // Deserialize GraphQL envelope
        let graphql_response: GraphQLResponse<R> =
            serde_json::from_str(&response_text).map_err(|e| {
                ApiError::Deserialization(format!(
                    "Failed to parse GraphQL response: {}. Body: {}",
                    e,
                    truncate_for_error(&response_text, 200)
                ))
            })?;

        // Extract data or convert errors
        graphql_response.into_result()
    }

    /// Truncate a string for error messages.
    fn truncate_for_error(s: &str, max_len: usize) -> &str {
        if s.len() <= max_len {
            s
        } else {
            &s[..max_len]
        }
    }
}

// Re-export SSR functions at module level
#[cfg(feature = "ssr")]
pub use ssr::*;

// ============================================================================
// GraphQL query/mutation string constants
// ============================================================================

/// Mutation: Verify a referral code.
pub const VERIFY_REFERRAL_MUTATION: &str = r#"
mutation VerifyReferralString($referralString: String!) {
    verifyReferralString(referralString: $referralString) {
        status
        ResponseCode
        affectedRows {
            uid
            username
            slug
            img
        }
    }
}
"#;

/// Mutation: Register a new user.
pub const REGISTER_MUTATION: &str = r#"
mutation Register($input: RegistrationInput!) {
    register(input: $input) {
        status
        ResponseCode
        userid
    }
}
"#;

/// Mutation: Verify a user account after registration.
pub const VERIFY_ACCOUNT_MUTATION: &str = r#"
mutation VerifyAccount($userId: ID!) {
    verifyAccount(userid: $userId) {
        status
        ResponseCode
    }
}
"#;

// ============================================================================
// Response wrapper types for GraphQL data field
// ============================================================================

/// Wrapper for the `verifyReferralString` mutation response.
///
/// The GraphQL response has shape:
/// ```json
/// { "data": { "verifyReferralString": { ... } } }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReferralData {
    pub verify_referral_string: crate::models::user::ReferralVerifyResponse,
}

/// Wrapper for the `register` mutation response.
#[derive(Debug, Deserialize)]
pub struct RegisterData {
    pub register: crate::models::user::RegisterResponse,
}

/// Wrapper for the `verifyAccount` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyAccountData {
    pub verify_account: crate::models::user::VerifyAccountResponse,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_serialization() {
        #[derive(Serialize)]
        struct TestVars {
            name: String,
        }

        let request = GraphQLRequest::new(
            "query Test($name: String!) { hello(name: $name) }",
            TestVars {
                name: "World".into(),
            },
        );

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""query":"query Test"#));
        assert!(json.contains(r#""variables":{"name":"World"}"#));
        assert!(!json.contains("operationName")); // Should be skipped when None
    }

    #[test]
    fn test_graphql_request_with_operation_name() {
        #[derive(Serialize)]
        struct EmptyVars {}

        let request =
            GraphQLRequest::with_operation("mutation DoThing { thing }", EmptyVars {}, "DoThing");

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""operationName":"DoThing""#));
    }

    #[test]
    fn test_graphql_response_success() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestData {
            value: i32,
        }

        let json = r#"{"data": {"value": 42}}"#;
        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        assert!(!response.has_errors());
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().value, 42);
    }

    #[test]
    fn test_graphql_response_with_errors() {
        #[derive(Debug, Deserialize)]
        struct TestData {
            #[allow(dead_code)]
            value: i32,
        }

        let json = r#"{
            "data": null,
            "errors": [
                {"message": "Something went wrong", "locations": [], "path": []}
            ]
        }"#;

        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        assert!(response.has_errors());
        assert_eq!(response.first_error_message(), Some("Something went wrong"));

        let result = response.into_result();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GraphQL(msg) => assert_eq!(msg, "Something went wrong"),
            _ => panic!("Expected GraphQL error"),
        }
    }

    #[test]
    fn test_graphql_response_empty() {
        #[derive(Debug, Deserialize)]
        struct TestData {}

        let json = r#"{"data": null}"#;
        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        let result = response.into_result();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unexpected(msg) => {
                assert!(msg.contains("neither data nor errors"));
            }
            _ => panic!("Expected Unexpected error"),
        }
    }

    #[test]
    fn test_verify_referral_data_deserialization() {
        let json = r#"{
            "verifyReferralString": {
                "status": "success",
                "ResponseCode": "11011",
                "affectedRows": [{
                    "uid": "abc-123",
                    "username": "testuser",
                    "slug": "12345",
                    "img": null
                }]
            }
        }"#;

        let data: VerifyReferralData = serde_json::from_str(json).unwrap();
        assert_eq!(data.verify_referral_string.status, "success");
        assert_eq!(data.verify_referral_string.response_code, "11011");
        assert!(data.verify_referral_string.affected_rows.is_some());
    }
}
