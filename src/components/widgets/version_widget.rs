//! Version info widget.

use leptos::prelude::*;

/// Widget showing app version information.
#[component]
pub fn VersionWidget() -> impl IntoView {
    // Version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");

    view! {
        <div class="widget widget-version">
            <div class="widget-inner widget-type-text">
                <span class="version-label">"Peer Web"</span>
                <span class="version-number">"v"{version}</span>
            </div>
        </div>
    }
}
