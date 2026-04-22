use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use async_graphql_axum::GraphQLRequest;
use axum::http::HeaderMap;
use axum::{Json, Router, extract::State, routing::post};
use tower_http::cors::{Any, CorsLayer};

pub mod filters;
pub mod guards;
pub mod routes;
pub mod schema;
pub mod seed;
pub mod state;
pub mod types;

// Re-export auth helpers for cross-module use
pub use schema::mutation::auth::{get_current_user, require_auth};

use schema::{AppSchema, build_schema};
use state::{MockState, SharedState};

use routes::upload::upload_post_handler;

/// Optional current-user identifier injected by auth extraction
#[derive(Clone, Debug)]
pub struct CurrentUser(pub Option<Uuid>);

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub schema: AppSchema,
    pub mock_state: SharedState,
}

/// GraphQL handler with auth context extraction
async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    let (current_user, roles_mask) = extract_bearer_user(&state, &headers).await;
    let mut request = req.into_inner();
    request = request.data(CurrentUser(current_user));
    if let Some(mask) = roles_mask {
        request = request.data(guards::UserRolesMask(mask));
    }
    state.schema.execute(request).await.into()
}

async fn extract_bearer_user(state: &AppState, headers: &HeaderMap) -> (Option<Uuid>, Option<u32>) {
    let auth_header = match headers.get("authorization") {
        Some(h) => h,
        None => return (None, None),
    };
    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => return (None, None),
    };
    let token = match auth_str.strip_prefix("Bearer ") {
        Some(t) => t,
        None => return (None, None),
    };
    let mock_state = state.mock_state.read().await;
    match mock_state.access_tokens.get(token) {
        Some(&uid) => {
            let roles_mask = mock_state.users.get(&uid).map(|u| u.role).unwrap_or(0);
            (Some(uid), Some(roles_mask))
        }
        None => (None, None),
    }
}

/// Reset handler — clears mutable state for test isolation
async fn reset_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut mock_state = state.mock_state.write().await;
    mock_state.reset();
    Json(serde_json::json!({ "status": "ok" }))
}

fn build_router(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/reset", post(reset_handler))
        .route("/upload-post", post(upload_post_handler))
        .layer(cors)
        .with_state(app_state)
}

/// Build the application router
pub fn app() -> Router {
    let mock_state: SharedState = Arc::new(RwLock::new(MockState::default()));
    let schema = build_schema(mock_state.clone());

    let app_state = AppState {
        schema: schema.clone(),
        mock_state,
    };

    build_router(app_state)
}

/// Build app with custom initial state (for testing)
pub fn app_with_state(mock_state: SharedState) -> Router {
    let schema = build_schema(mock_state.clone());

    let app_state = AppState {
        schema: schema.clone(),
        mock_state,
    };

    build_router(app_state)
}
