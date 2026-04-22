//! Right-rail wrapper + the `StandardRightRail` widget-stack preset.

use leptos::prelude::*;

use crate::components::widgets::{MainMenu, ProfileWidget, VersionWidget};

/// Wrap page-specific right-sidebar content in the standard
/// `<aside>` + `inner-scroll` markup.
#[component]
pub fn RightRail(
    /// Slug appended to the class list (e.g. `"wallet"` →
    /// `right-sidebar-wallet`).
    #[prop(into)]
    slug: String,
    children: Children,
) -> impl IntoView {
    let class = format!("right-sidebar right-sidebar-{slug}");

    view! {
        <aside class=class>
            <div class="inner-scroll">
                {children()}
            </div>
        </aside>
    }
}

/// Convenience composition: `ProfileWidget` → `MainMenu` →
/// `NewPostButton` → `VersionWidget`.
///
/// Used verbatim by Wallet, Chat, Settings, Referral Board, and
/// Version History today.
#[component]
pub fn StandardRightRail(
    /// Slug appended to the class list (e.g. `"wallet"` →
    /// `right-sidebar-wallet`).
    #[prop(into)]
    slug: String,
) -> impl IntoView {
    view! {
        <RightRail slug=slug>
            <ProfileWidget/>
            <MainMenu/>
            <NewPostButton/>
            <VersionWidget/>
        </RightRail>
    }
}

/// "New Post" sidebar button. The widget chrome (class names) matches
/// the per-page copies it replaces in Wallet / Settings / Version
/// History / My Ads / Referral Board.
#[component]
fn NewPostButton() -> impl IntoView {
    view! {
        <div class="new-post-widget">
            <a href="/newpost" class="new-post-btn">
                <i class="peer-icon peer-icon-plus"/>
                <span>"New Post"</span>
            </a>
        </div>
    }
}
