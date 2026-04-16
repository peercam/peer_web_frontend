use mock_backend::state::MockState;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn default_shared_state() -> Arc<RwLock<MockState>> {
    Arc::new(RwLock::new(MockState::default()))
}
