use leptos::prelude::*;

/// Back button for multi-step navigation.
///
/// On step 1: renders as a link to `/login` (full page navigation).
/// On step 2: calls `on_back` callback to return to step 1.
/// On step 3: hidden entirely.
#[component]
pub fn BackButton(
    /// Whether the button is currently visible.
    visible: Signal<bool>,
    /// If `Some(url)`, renders as a link to that URL.
    /// If `None`, renders as a button that calls `on_back`.
    href: Signal<Option<String>>,
    /// Callback invoked when back is clicked (only when `href` is `None`).
    on_back: Callback<()>,
) -> impl IntoView {
    view! {
        <a
            href=move || href.get().unwrap_or_default()
            class="btn btn-secondary back-btn"
            id="backBtn"
            style:display=move || if visible.get() { "flex" } else { "none" }
            on:click=move |ev| {
                if href.get().is_none() {
                    ev.prevent_default();
                    on_back.run(());
                }
            }
        >
            <span aria-hidden="true">
                <i class="peer-icon medium_font peer-icon-arrow-left"></i>
            </span>
            "Back"
        </a>
    }
}
