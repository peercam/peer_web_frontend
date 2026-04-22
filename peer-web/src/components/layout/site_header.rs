//! Standard `<header>` for authenticated pages.
//!
//! Two spellings exist in production today (see Open Question 3 in
//! [`docs/plans/layout/layout-shell-implementation.md`]):
//!
//! - `Hyphen` (default) → `class="site-header header-{modifier}"` — the
//!   broadly-styled form used by Wallet, My Ads, Referral Board, Settings,
//!   Version History.
//! - `Underscore` → `class="site_header"` — used by Chat (scoped under
//!   `.chat` in `chat.scss`), Profile, View Profile, Peer Shop.
//!
//! This component is wired up by Pass-2 callers. Pass-1 pages keep their
//! existing inline `<header>` and pass it as a child of [`SiteShell`].

use leptos::prelude::*;

/// Class spelling for the rendered `<header>`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum HeaderSpelling {
    /// `class="site-header header-{modifier}"` — broadly-styled form.
    #[default]
    Hyphen,
    /// `class="site_header"` — Chat / Profile / View Profile / Peer Shop.
    Underscore,
}

/// Standard authenticated-page header.
#[component]
pub fn SiteHeader(
    /// Page title displayed in the centre column.
    #[prop(into)]
    title: String,
    /// Per-page modifier appended after the base class
    /// (e.g. `"wallet"` → `header-wallet`). Required for the
    /// [`HeaderSpelling::Hyphen`] spelling, ignored for
    /// [`HeaderSpelling::Underscore`].
    #[prop(into, optional)]
    modifier: Option<String>,
    /// Class spelling. Default: [`HeaderSpelling::Hyphen`].
    #[prop(default = HeaderSpelling::Hyphen)]
    spelling: HeaderSpelling,
    /// Optional peer-icon class for the title prefix
    /// (e.g. `"peer-icon-wallet-filled"`).
    #[prop(into, optional)]
    icon: Option<String>,
    /// Optional right-aligned actions slot.
    #[prop(optional)]
    actions: Option<Children>,
    /// If true, render the home-link logo on the left. Default: `true`.
    #[prop(default = true)]
    show_logo: bool,
) -> impl IntoView {
    let class = match spelling {
        HeaderSpelling::Hyphen => match modifier.as_deref() {
            Some(m) if !m.is_empty() => format!("site-header header-{m}"),
            _ => "site-header".to_string(),
        },
        HeaderSpelling::Underscore => "site_header".to_string(),
    };

    let icon_view = icon
        .filter(|c| !c.is_empty())
        .map(|c| view! { <i class=format!("peer-icon {c}")/> });

    let actions_view = actions.map(|children| {
        view! { <div class="header-actions">{children()}</div> }
    });

    view! {
        <header class=class>
            <div class="site_header_inner">
                {show_logo.then(|| view! {
                    <div class="logo_box">
                        <a href="/" class="logo" aria-label="Peer Network Home">
                            <img src="/img/peer-logo.png" alt="Peer Network" class="logo-img"/>
                        </a>
                    </div>
                })}
                <div class="page-title">
                    <h1>{icon_view}{title}</h1>
                </div>
                {actions_view}
            </div>
        </header>
    }
}
