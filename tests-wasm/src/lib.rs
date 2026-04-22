//! Headless-browser test harness for Leptos components.
//!
//! See `README.md` for usage. This module is the equivalent of
//! React Testing Library's `render` / `fireEvent.input` / `screen`,
//! tailored to Leptos 0.8 and `wasm-bindgen-test`.

#![cfg(target_arch = "wasm32")]

pub mod harness;

#[cfg(test)]
mod tests;
