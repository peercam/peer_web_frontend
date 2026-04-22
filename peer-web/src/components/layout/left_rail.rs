//! Left-rail wrapper: `<aside class="left-sidebar left-sidebar-{slug}">`.

use leptos::prelude::*;

/// Wrap page-specific left-sidebar content in the standard
/// `<aside>` + `inner-scroll` markup.
#[component]
pub fn LeftRail(
    /// Slug appended to the class list (e.g. `"wallet"` →
    /// `left-sidebar-wallet`).
    #[prop(into)]
    slug: String,
    children: Children,
) -> impl IntoView {
    let class = format!("left-sidebar left-sidebar-{slug}");

    view! {
        <aside class=class>
            <div class="inner-scroll">
                {children()}
            </div>
        </aside>
    }
}
