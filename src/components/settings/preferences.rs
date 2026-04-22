//! Preferences settings tab component (stub).
//!
//! Placeholder for future user preferences.

use leptos::prelude::*;

/// Preferences Settings tab — placeholder.
#[component]
pub fn PreferencesSettings() -> impl IntoView {
    view! {
        <div id="preferences-settings" class="setting-content">
            <div class="settings-stub">
                <i class="peer-icon peer-icon-settings"></i>
                <h3 class="xl_font_size">"Preferences"</h3>
                <p class="md_font_size txt-color-gray">
                    "Preference settings are coming soon."
                </p>
                <p class="sm_font_size txt-color-gray">
                    "You'll be able to customize your app experience, language, and display options."
                </p>
            </div>
        </div>
    }
}
