
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use peer_web::app::*;
    use peer_web::server::download::{download_handler, DownloadConfig};

    // Log GraphQL endpoint configuration
    let graphql_endpoint = std::env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string());
    println!("GraphQL endpoint: {}", graphql_endpoint);

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let download_config = DownloadConfig::from_env();

    let app = Router::new()
        // Register /download BEFORE leptos_routes so it cannot be shadowed by a
        // future Leptos page at the same path. Uses its own state type.
        .route(
            "/download",
            axum::routing::get(download_handler).with_state(download_config),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
