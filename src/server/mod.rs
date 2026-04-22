//! Server-only HTTP routes that live outside the Leptos router.
//!
//! Unlike `api::*` (GraphQL server functions called from WASM), this module hosts
//! plain Axum handlers that are mounted directly on the main router. Currently
//! only the `/download` force-download media proxy lives here.

pub mod download;
