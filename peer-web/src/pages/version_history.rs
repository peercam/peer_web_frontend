//! Version History page.
//!
//! Displays release notes and changelog loaded from a static JSON file.
//! Two-panel layout: version list (left) + version details (right).

use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::version::get_version_releases;
use crate::components::auth_guard::AuthGuard;
use crate::components::version_history::version_detail::VersionDetail;
use crate::components::version_history::version_list::VersionList;
use crate::components::widgets::{MainMenu, ProfileWidget, VersionWidget};

/// Version History page — auth-guarded, two-panel layout.
#[component]
pub fn VersionHistoryPage() -> impl IntoView {
    let selected_idx = RwSignal::new(Some(0usize));

    let versions_resource = Resource::new(|| (), |_| async move { get_version_releases().await });

    view! {
        <Title text="Version History - Peer Network"/>
        <AuthGuard>
            <div id="version_history" class="site_layout version-history-layout">
                <VersionHistoryHeader/>

                <aside class="left-sidebar left-sidebar-profile">
                    <div class="inner-scroll">
                        <div class="profile-back-button">
                            <a href="/settings" class="button btn-transparent">
                                "Back to Settings"
                            </a>
                        </div>
                    </div>
                </aside>

                <main class="site-main site_main_versionHistory">
                    <div class="setting-layout">
                        <Suspense fallback=move || {
                            view! { <p class="loading-text">"Loading version history…"</p> }
                        }>
                            {move || {
                                versions_resource
                                    .get()
                                    .map(|result| match result {
                                        Ok(versions) => {
                                            view! {
                                                <VersionList
                                                    versions=versions.clone()
                                                    selected_idx=selected_idx
                                                />
                                                <VersionDetail
                                                    versions=versions
                                                    selected_idx=selected_idx
                                                />
                                            }
                                                .into_any()
                                        }
                                        Err(e) => {
                                            let msg = e.to_string();
                                            view! {
                                                <div class="error-state">
                                                    <p>"Failed to load version history."</p>
                                                    <p class="error-detail">{msg}</p>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    })
                            }}
                        </Suspense>
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

/// Version History page header with logo and title.
#[component]
fn VersionHistoryHeader() -> impl IntoView {
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
