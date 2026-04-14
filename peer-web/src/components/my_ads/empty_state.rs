//! Empty state component for when the user has no advertisements.

use leptos::prelude::*;

/// Empty state shown when user has no advertisements.
#[component]
pub fn EmptyState() -> impl IntoView {
    view! {
        <div class="empty-state-container">
            <p class="empty-state-message">
                "You haven't promoted any posts yet. Start your first promotion to see statistics"
            </p>
            <a href="/profile" class="button btn-white">
                "Take me to my posts"
            </a>
        </div>
    }
}
