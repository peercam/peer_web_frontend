// peer-web/tests/mock_connectivity.rs
#[tokio::test]
async fn mock_backend_is_reachable() {
    let endpoint = std::env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string());
    
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
