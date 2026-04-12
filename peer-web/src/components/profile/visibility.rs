//! Content visibility overlays and badges.
//!
//! Components for handling hidden content overlays and illegal content badges.

use leptos::prelude::*;

/// Overlay shown for hidden/sensitive content.
///
/// Displays a semi-transparent overlay with a "View anyway" button that
/// allows users to reveal the hidden content.
#[component]
pub fn HiddenOverlay(
    /// Callback when user chooses to view the hidden content.
    on_view: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="hidden-content-overlay">
            <div class="overlay-content">
                <i class="peer-icon peer-icon-eye-off"></i>
                <h3>"Sensitive Content"</h3>
                <p>"This content may be sensitive for some viewers."</p>
                <button
                    class="button btn-blue view-anyway"
                    on:click=move |_| on_view.run(())
                >
                    "View anyway"
                </button>
            </div>
        </div>
    }
}

/// Badge indicating illegal content has been removed.
///
/// Replaces the profile picture/content area when a profile is marked as illegal.
#[component]
pub fn IllegalProfileBadge() -> impl IntoView {
    view! {
        <div class="illegal-profile-badge">
            <i class="peer-icon peer-icon-ban"></i>
            <span>"Content removed"</span>
        </div>
    }
}

/// Badge indicating a user/content has been reported.
#[component]
pub fn ReportedBadge() -> impl IntoView {
    view! {
        <span class="reported-badge" title="This content has been reported">
            <i class="peer-icon peer-icon-flag"></i>
        </span>
    }
}
