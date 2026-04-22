// tests/mock_connectivity.rs
//!
//! Sanity check: the `mock_backend` crate is embedded as a library and
//! serves a minimal GraphQL request in-process. No external service required.

#![cfg(feature = "ssr")]

mod common;

#[tokio::test]
async fn mock_backend_is_reachable() {
    let endpoint = common::mock_graphql_endpoint().await;

    let client = reqwest::Client::new();
    let res = client
        .post(&endpoint)
        .json(&serde_json::json!({
            "query": "{ _health }"
        }))
        .send()
        .await
        .expect("Failed to reach mock backend");

    assert!(res.status().is_success());
}
