use mock_backend::app;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    println!("Mock Peer backend running at http://localhost:{port}/graphql");
    println!("   POST /graphql  — GraphQL endpoint");
    println!("   POST /reset    — Reset state for testing");

    axum::serve(listener, app()).await.expect("Server error");
}
