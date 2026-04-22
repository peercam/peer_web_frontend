//! Shared test helpers for integration tests.
//!
//! Spins up the GraphQL `mock_backend` **in-process** on an ephemeral
//! loopback port. Tests no longer require a separately-running
//! `cargo run -p mock_backend` service.
//!
//! Each integration test binary (each `tests/*.rs` file) starts its own
//! single shared instance on first use and reuses it for all tests in
//! that binary.

#![cfg(feature = "ssr")]
#![allow(dead_code)]

use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

static MOCK_ENDPOINT: OnceLock<String> = OnceLock::new();

/// Serializes tests that mutate the shared chat state on the in-process
/// mock backend (the seeded private/group chats are global to the
/// backend instance). Tokio's default test runner executes `#[tokio::test]`
/// bodies concurrently; without this lock a `sendChatMessage` from one
/// test can race a `markChatRead` in another and bump the unread count
/// back up. Acquire via [`chat_state_guard`] for the full test body.
static CHAT_STATE_LOCK: Mutex<()> = Mutex::const_new(());

/// Global lock over the `GRAPHQL_ENDPOINT` env var. Tests that rely on the
/// in-process mock backend acquire a read guard via [`endpoint_read_guard`];
/// tests that mutate the env var acquire a write guard via
/// [`endpoint_write_guard`]. This prevents the race between the
/// "wrong endpoint" error test and concurrent happy-path tests.
static ENDPOINT_LOCK: RwLock<()> = RwLock::const_new(());

/// Start the mock backend (once per test process) on its own dedicated
/// runtime thread, and return the `/graphql` endpoint URL. Also sets
/// `GRAPHQL_ENDPOINT` so that the production
/// `peer_web::api::graphql::get_endpoint()` helper picks up the ephemeral
/// address.
///
/// The server must live on a *separate* runtime because each
/// `#[tokio::test]` creates and tears down its own runtime; spawning the
/// server inside the first test would kill it when that test ends.
pub async fn mock_graphql_endpoint() -> String {
    MOCK_ENDPOINT.get_or_init(start_mock_backend_once).clone()
}

fn start_mock_backend_once() -> String {
    // Bind synchronously on a standard listener so we know the port before
    // handing ownership to the dedicated runtime thread.
    let std_listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("bind mock_backend listener");
    std_listener
        .set_nonblocking(true)
        .expect("set_nonblocking on listener");
    let addr = std_listener.local_addr().expect("local_addr");
    let endpoint = format!("http://{}/graphql", addr);

    std::thread::Builder::new()
        .name("mock-backend".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build mock-backend runtime");
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(std_listener)
                    .expect("convert std listener to tokio");
                let app = mock_backend::app();
                let _ = axum::serve(listener, app).await;
            });
        })
        .expect("spawn mock-backend thread");

    // SAFETY: set_var is unsafe on 2024 edition. We set it exactly once,
    // inside `OnceLock::get_or_init`, before any test issues a GraphQL
    // request through `get_endpoint()`.
    unsafe {
        std::env::set_var("GRAPHQL_ENDPOINT", &endpoint);
    }

    endpoint
}

/// Acquire a shared read lock on `GRAPHQL_ENDPOINT` for a normal happy-path
/// test. Hold it for the duration of any GraphQL request you make.
pub async fn endpoint_read_guard() -> RwLockReadGuard<'static, ()> {
    ENDPOINT_LOCK.read().await
}

/// Acquire an exclusive write lock on `GRAPHQL_ENDPOINT` for a test that
/// deliberately mutates the env var (e.g. pointing at a dead port to
/// exercise error paths). Hold it for the full mutate/restore window.
pub async fn endpoint_write_guard() -> RwLockWriteGuard<'static, ()> {
    ENDPOINT_LOCK.write().await
}

/// Acquire an exclusive lock that serializes tests mutating the shared
/// chat state (the seeded chats, messages, read cursors). Hold it for
/// the full test body.
pub async fn chat_state_guard() -> MutexGuard<'static, ()> {
    CHAT_STATE_LOCK.lock().await
}
