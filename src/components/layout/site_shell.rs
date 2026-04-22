//! Outer page chrome: `<div class="site_layout">` + auto-appended
//! [`MobileFooter`].
//!
//! `SiteShell` is intentionally a thin wrapper. Pages pass their existing
//! header / sidebar / main markup as `children`; the shell only owns the
//! root `<div>` and the trailing `<MobileFooter/>`. This keeps the Pass-1
//! migration to a near-trivial diff per page (see Phase 2 of
//! [`docs/plans/layout/layout-shell-implementation.md`]).

use leptos::prelude::*;

use super::mobile_footer::MobileFooter;

/// Compose the standard authenticated page shell.
#[component]
pub fn SiteShell(
    /// Value for the root element's `id` attribute (e.g. `"wallet-page"`).
    #[prop(into)]
    id: String,
    /// Optional class(es) appended after `site_layout`
    /// (e.g. `"wallet-layout"`). May contain multiple space-separated
    /// classes — `peer_shop` uses this for its leading
    /// `view-peer-shop profile-layout` pair.
    #[prop(into, optional)]
    modifier: Option<String>,
    /// Page contents (header, rails, main, …).
    children: Children,
    /// If true, append the shared [`MobileFooter`]. Default: `true`.
    #[prop(default = true)]
    mobile_footer: bool,
) -> impl IntoView {
    let class = match modifier {
        Some(m) if !m.is_empty() => format!("site_layout {m}"),
        _ => "site_layout".to_string(),
    };

    view! {
        <div id=id class=class>
            {children()}
            {mobile_footer.then(|| view! { <MobileFooter/> })}
        </div>
    }
}
