use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mock_backend::{app, app_with_state, state::MockState};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

/// Core GraphQL request — supports optional shared state and optional auth token.
pub async fn graphql_request(
    state: Option<&Arc<RwLock<MockState>>>,
    query: &str,
    token: Option<&str>,
) -> Value {
    let app = match state {
        Some(s) => app_with_state(s.clone()),
        None => app(),
    };

    let body = json!({ "query": query });

    let mut builder = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("Content-Type", "application/json");

    if let Some(t) = token {
        builder = builder.header("Authorization", format!("Bearer {}", t));
    }

    let request = builder
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body_bytes).unwrap()
}

/// Stateless, no auth.
pub async fn graphql(query: &str) -> Value {
    graphql_request(None, query, None).await
}

/// Shared state, no auth.
pub async fn graphql_stateful(state: &Arc<RwLock<MockState>>, query: &str) -> Value {
    graphql_request(Some(state), query, None).await
}

/// Shared state + Bearer token.
pub async fn graphql_with_auth(
    state: &Arc<RwLock<MockState>>,
    query: &str,
    token: &str,
) -> Value {
    graphql_request(Some(state), query, Some(token)).await
}

/// Helper for post action mutations.
pub async fn do_post_action(
    state: &Arc<RwLock<MockState>>,
    token: &str,
    postid: &str,
    action: &str,
) -> Value {
    graphql_with_auth(
        state,
        &format!(
            r#"
        mutation {{
            resolvePostAction(postid: "{postid}", action: {action}) {{
                status RequestId ResponseCode ResponseMessage
            }}
        }}
    "#
        ),
        token,
    )
    .await
}
