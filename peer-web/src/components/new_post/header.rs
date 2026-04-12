//! New post header component.

use leptos::prelude::*;

use crate::pages::new_post::NewPostContext;

/// Header for the new post page.
#[component]
pub fn NewPostHeader() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    view! {
        <header class="site-header site-header-new-post">
            <div class="header-content">
                <a href="/dashboard" class="header-logo">
                    <img src="/svg/logo.svg" alt="Peer" class="logo-img"/>
                </a>
                <h1 class="header-title">
                    {move || ctx.content_type.get().display_name()}
                </h1>
            </div>
        </header>
    }
}
