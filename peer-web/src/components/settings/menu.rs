//! Settings tab navigation menu component.

use leptos::prelude::*;

use crate::pages::settings::SettingsTab;
use crate::state::auth::use_auth;

/// Settings tab navigation menu.
///
/// Displays tab buttons, logout, and deactivate profile links.
#[component]
pub fn SettingsMenu(
    /// The currently active tab.
    active_tab: RwSignal<SettingsTab>,
    /// Callback when a tab is selected.
    on_tab_change: impl Fn(SettingsTab) + 'static + Copy,
) -> impl IntoView {
    let auth = use_auth();
    let show_logout_modal = RwSignal::new(false);

    let tabs = vec![
        (SettingsTab::Profile, "Profile Settings"),
        (SettingsTab::Notifications, "Notification Settings"),
        (SettingsTab::Preferences, "Preferences"),
        (SettingsTab::Content, "Content Settings"),
    ];

    view! {
        <div class="setting-menu">
            <ul>
                {tabs.into_iter().map(|(tab, label)| {
                    view! {
                        <li
                            class:active=move || active_tab.get() == tab
                            on:click=move |_| on_tab_change(tab)
                        >
                            <a href="#" class="md_font_size"
                                on:click=|e| e.prevent_default()
                            >
                                {label}
                            </a>
                        </li>
                    }
                }).collect_view()}

                // Logout button
                <li class="not-menu-item">
                    <a href="#" class="md_font_size red-btn"
                        on:click=move |e| {
                            e.prevent_default();
                            show_logout_modal.set(true);
                        }
                    >
                        "Log Out"
                    </a>
                </li>

                // Deactivate button
                <li class="not-menu-item">
                    <a href="#" class="md_font_size red-btn">
                        "Deactivate profile"
                    </a>
                </li>
            </ul>

            // Logout confirmation modal
            <Show when=move || show_logout_modal.get()>
                <LogoutModal
                    on_cancel=move || show_logout_modal.set(false)
                    on_confirm=move || {
                        auth.logout_action.dispatch(());
                        show_logout_modal.set(false);
                    }
                />
            </Show>
        </div>
    }
}

/// Logout confirmation modal.
#[component]
fn LogoutModal(
    on_cancel: impl Fn() + 'static,
    on_confirm: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="modal-overlay">
            <div class="logOut-pop">
                <img src="/svg/Union.svg" alt="logout"/>
                <p class="xl_font_size bold">"Are you sure you want to log out?"</p>
                <div class="button-row">
                    <button class="btn-white" on:click=move |_| on_cancel()>"Cancel"</button>
                    <button class="btn-red-transparent" on:click=move |_| on_confirm()>"Log Out"</button>
                </div>
            </div>
        </div>
    }
}
