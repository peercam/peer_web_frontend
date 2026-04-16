//! Settings page.
//!
//! Multi-tab settings interface for managing profile, preferences,
//! content settings, and account actions.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::profile::get_profile;
use crate::components::auth_guard::AuthGuard;
use crate::components::settings::{
    ContentSettings, DeactivateAccountPanel, NotificationSettings, PreferencesSettings, ProfileSettings, SettingsMenu,
};
use crate::components::widgets::{MainMenu, ProfileWidget, VersionWidget};

/// Active settings tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Profile,
    Notifications,
    Preferences,
    Content,
    Deactivate,
}

/// Settings page — auth-guarded, tabbed layout.
#[component]
pub fn SettingsPage() -> impl IntoView {
    let active_tab = RwSignal::new(SettingsTab::Profile);

    // Load current user profile for the profile settings tab
    let profile_resource = Resource::new(|| (), |_| async move { get_profile(None, None).await });

    view! {
        <Title text="Settings - Peer Network"/>
        <AuthGuard>
            <div id="edit-profile" class="site_layout settings-layout">
                <SettingsHeader/>

                <aside class="left-sidebar left-sidebar-profile">
                    <div class="inner-scroll">
                        <div class="profile-back-button">
                            <a href="/profile" class="button btn-transparent">
                                "Back to Profile"
                            </a>
                        </div>
                    </div>
                </aside>

                <main class="site-main site-main-edit-profile">
                    <div class="setting-layout">
                        <SettingsMenu
                            active_tab=active_tab
                            on_tab_change=move |tab| active_tab.set(tab)
                            on_deactivate=move || active_tab.set(SettingsTab::Deactivate)
                        />

                        <div class="settings-content-wrapper">
                            {move || {
                                let tab = active_tab.get();
                                match tab {
                                    SettingsTab::Profile => {
                                        view! { <ProfileSettings profile_resource=profile_resource/> }
                                            .into_any()
                                    }
                                    SettingsTab::Notifications => {
                                        view! { <NotificationSettings/> }.into_any()
                                    }
                                    SettingsTab::Preferences => {
                                        view! { <PreferencesSettings/> }.into_any()
                                    }
                                    SettingsTab::Content => {
                                        view! { <ContentSettings/> }.into_any()
                                    }
                                    SettingsTab::Deactivate => {
                                        view! {
                                            <DeactivateAccountPanel
                                                on_back=move || active_tab.set(SettingsTab::Profile)
                                            />
                                        }.into_any()
                                    }
                                }
                            }}
                        </div>
                    </div>
                </main>

                <aside class="right-sidebar right-sidebar-edit-profile">
                    <div class="inner-scroll">
                        <ProfileWidget/>
                        <MainMenu/>
                        <NewPostButton/>
                        <VersionWidget/>
                    </div>
                </aside>

                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}

/// Settings page header with logo and title.
#[component]
fn SettingsHeader() -> impl IntoView {
    view! {
        <header class="site-header header-profile">
            <img class="logo" src="/svg/logo_sw.svg" alt="Peer Network"/>
            <h1>"Settings"</h1>
        </header>
    }
}

/// New post button for sidebar.
#[component]
fn NewPostButton() -> impl IntoView {
    view! {
        <div class="new-post-widget">
            <a href="/newpost" class="new-post-btn">
                <i class="peer-icon peer-icon-plus"></i>
                <span>"New Post"</span>
            </a>
        </div>
    }
}

/// Mobile navigation footer.
#[component]
fn MobileFooter() -> impl IntoView {
    view! {
        <footer class="mobile-footer">
            <nav class="mobile-nav">
                <a href="/dashboard" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-home"></i>
                </a>
                <a href="/chat" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-chat"></i>
                </a>
                <a href="/newpost" class="mobile-nav-item mobile-nav-item-new">
                    <i class="peer-icon peer-icon-plus"></i>
                </a>
                <a href="/wallet" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-wallet"></i>
                </a>
                <a href="/profile" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-user"></i>
                </a>
            </nav>
        </footer>
    }
}
