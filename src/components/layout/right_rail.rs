//! Right-rail wrapper + the `StandardRightRail` widget-stack preset.

use leptos::prelude::*;

use crate::components::widgets::{AddPostButton, MainMenu, ProfileWidget, VersionWidget};

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
/// `AddPostButton` → `VersionWidget`.
///
/// Used verbatim by Wallet, Chat, Settings, Referral Board, and
/// Version History today. The button widget is the shared
/// [`crate::components::widgets::AddPostButton`] (matching the legacy
/// `template-parts/sidebars/widget-add-new-post.php` chrome).
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
            <AddPostButton/>
            <VersionWidget/>
        </RightRail>
    }
}
