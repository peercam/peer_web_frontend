#![recursion_limit = "512"]

pub mod app;
pub mod api;
pub mod components;
pub mod hooks;
pub mod models;
pub mod pages;
pub mod state;
pub mod utils;

#[cfg(feature = "ssr")]
pub mod server;

// Re-export commonly used types
pub use models::common::ApiError;
pub use models::user::{RegistrationInput, RegisterResponse, ReferralVerifyResponse, VerifyAccountResponse};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
