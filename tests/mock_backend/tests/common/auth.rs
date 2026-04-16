use mock_backend::state::MockState;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::client::graphql_stateful;

pub async fn login_as(state: &Arc<RwLock<MockState>>, email: &str, password: &str) -> String {
    let res = graphql_stateful(
        state,
        &format!(
            r#"
        mutation {{
            login(email: "{email}", password: "{password}") {{
                accessToken
            }}
        }}
    "#
        ),
    )
    .await;
    res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn login_default(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "test@peer.com", "TestPass123").await
}

pub async fn login_alice(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "alice@peer.com", "AlicePass123").await
}

pub async fn login_bob(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "bob@peer.com", "BobPass123").await
}

pub async fn login_admin(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "admin@peerapp.de", "Admin1234").await
}

pub async fn login_moderator(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "mod@peerapp.de", "Mod1234").await
}
