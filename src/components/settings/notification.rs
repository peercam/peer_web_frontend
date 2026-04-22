//! Notification settings tab component (stub).
//!
//! Placeholder for future notification preferences.

use leptos::prelude::*;

/// Notification Settings tab — placeholder.
#[component]
pub fn NotificationSettings() -> impl IntoView {
    view! {
        <div id="notification-settings" class="setting-content">
            <div class="settings-stub">
                <i class="peer-icon peer-icon-bell"></i>
                <h3 class="xl_font_size">"Notification Settings"</h3>
                <p class="md_font_size txt-color-gray">
                    "Notification preferences are coming soon."
                </p>
                <p class="sm_font_size txt-color-gray">
                    "You'll be able to customize which notifications you receive and how you receive them."
                </p>
            </div>
        </div>
    }
}
