//! Chat page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;

use crate::components::auth_guard::AuthGuard;
use crate::components::chat::{ChatContainer, ChatList};
use crate::components::layout::SiteShell;
use crate::components::widgets::{AddPostButton, MainMenu, ProfileWidget, VersionWidget};
use crate::state::chat::{ChatContext, load_chats, provide_chat_context, use_chat};
#[cfg(target_arch = "wasm32")]
use crate::state::chat::{poll_active_chat, refresh_chat_list};

// Poll intervals (milliseconds). The plan specifies ~15s for the chat
// list and ~5s for the active chat; both mirror the ADR defaults.
#[cfg(target_arch = "wasm32")]
const CHAT_LIST_POLL_INTERVAL_MS: u64 = 15_000;
#[cfg(target_arch = "wasm32")]
const CHAT_ACTIVE_POLL_INTERVAL_MS: u64 = 5_000;

/// Main chat page.
#[component]
pub fn ChatPage() -> impl IntoView {
    // Provide chat context for all child components.
    let ctx = provide_chat_context();

    // Initial load.
    Effect::new(move |_| {
        spawn_local(async move {
            load_chats(ctx).await;
        });
    });

    // Start polling timers and a visibility-change listener.
    start_polling(ctx);

    view! {
        <Title text="Chat - Peer Network"/>
        <AuthGuard>
            <SiteShell id="chat">
                <ChatHeader/>
                <LeftSidebar/>
                <main class="site-main site-main-chats">
                    <div class="main-chat">
                        <ChatList/>
                        <ChatContainer/>
                    </div>
                </main>
                <RightSidebar/>
            </SiteShell>
        </AuthGuard>
    }
}

/// Start list + active-chat poll timers. They pause while the browser
/// tab is hidden and resume (with an immediate catch-up) on `visible`.
fn start_polling(ctx: ChatContext) {
    #[cfg(target_arch = "wasm32")]
    {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        let paused = Rc::new(Cell::new(false));

        // Chat-list timer.
        let paused_list = paused.clone();
        leptos::leptos_dom::helpers::set_interval(
            move || {
                if paused_list.get() {
                    return;
                }
                spawn_local(async move {
                    refresh_chat_list(ctx).await;
                });
            },
            Duration::from_millis(CHAT_LIST_POLL_INTERVAL_MS),
        );

        // Active-chat timer.
        let paused_active = paused.clone();
        leptos::leptos_dom::helpers::set_interval(
            move || {
                if paused_active.get() {
                    return;
                }
                spawn_local(async move {
                    poll_active_chat(ctx).await;
                });
            },
            Duration::from_millis(CHAT_ACTIVE_POLL_INTERVAL_MS),
        );

        // visibilitychange listener → pause/resume.
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;

        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            let doc_for_closure = document.clone();
            let paused_cb = paused;
            let closure = Closure::wrap(Box::new(move || {
                let hidden = doc_for_closure.hidden();
                paused_cb.set(hidden);
                if !hidden {
                    // Immediate catch-up on return.
                    spawn_local(async move {
                        refresh_chat_list(ctx).await;
                        poll_active_chat(ctx).await;
                    });
                }
            }) as Box<dyn Fn()>);

            let _ = document.add_event_listener_with_callback(
                "visibilitychange",
                closure.as_ref().unchecked_ref(),
            );
            // Leak the closure so it lives for the page's lifetime.
            closure.forget();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // No polling on the server; SSR renders the initial snapshot only.
        let _ = ctx;
    }
}

/// Chat page header.
#[component]
fn ChatHeader() -> impl IntoView {
    view! {
        <header class="site_header">
            <div class="site_header_inner">
                <div class="logo_box">
                    <a href="/" class="logo" aria-label="Peer Network Home">
                        <img src="/img/peer-logo.png" alt="Peer Network" class="logo-img"/>
                    </a>
                </div>
                <div class="page-title">
                    <h1>"Chat"</h1>
                </div>
                <div class="header-actions">
                    <a href="/notifications" class="icon-btn" aria-label="Notifications">
                        <i class="peer-icon peer-icon-bell"/>
                    </a>
                </div>
            </div>
        </header>
    }
}

/// Left sidebar — search input wired to `ctx.search_query`.
#[component]
fn LeftSidebar() -> impl IntoView {
    let ctx = use_chat();

    let on_input = move |ev| {
        ctx.search_query.set(event_target_value(&ev));
    };

    view! {
        <aside class="left-sidebar left-sidebar-chats">
            <div class="inner-scroll">
                <div class="widget">
                    <div class="widget-inner">
                        <div class="search-box">
                            <i class="peer-icon peer-icon-search"/>
                            <input
                                type="text"
                                placeholder="Search chats..."
                                class="search-input"
                                prop:value=move || ctx.search_query.get()
                                on:input=on_input
                                aria-label="Search chats"
                            />
                        </div>
                    </div>
                </div>
            </div>
        </aside>
    }
}

/// Right sidebar with widgets.
#[component]
fn RightSidebar() -> impl IntoView {
    view! {
        <aside class="right-sidebar right-sidebar-chats">
            <div class="inner-scroll">
                <ProfileWidget/>
                <MainMenu/>
                <AddPostButton/>
                <VersionWidget/>
            </div>
        </aside>
    }
}
