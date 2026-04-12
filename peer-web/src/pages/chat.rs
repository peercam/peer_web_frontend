//! Chat page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;

use crate::components::auth_guard::AuthGuard;
use crate::components::chat::{ChatContainer, ChatList};
use crate::components::widgets::{AddPostButton, MainMenu, ProfileWidget, VersionWidget};
use crate::state::chat::{load_chats, provide_chat_context};

/// Main chat page.
///
/// Displays the chat interface with:
/// - Chat list sidebar (private/group tabs)
/// - Chat container (messages, input)
/// - Right sidebar with widgets
///
/// Requires authentication.
#[component]
pub fn ChatPage() -> impl IntoView {
    // Provide chat context for all child components
    let ctx = provide_chat_context();

    // Load chats on mount
    Effect::new(move |_| {
        spawn_local(async move {
            load_chats(ctx).await;
        });
    });

    view! {
        <Title text="Chat - Peer Network"/>
        <AuthGuard>
            <div id="chat" class="site_layout">
                <ChatHeader/>
                <LeftSidebar/>
                <main class="site-main site-main-chats">
                    <div class="main-chat">
                        <ChatList/>
                        <ChatContainer/>
                    </div>
                </main>
                <RightSidebar/>
                <MobileFooter/>
            </div>
        </AuthGuard>
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

/// Left sidebar (search).
#[component]
fn LeftSidebar() -> impl IntoView {
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

/// Mobile navigation footer.
#[component]
fn MobileFooter() -> impl IntoView {
    view! {
        <footer class="mobile-footer">
            <nav class="mobile-nav">
                <a href="/dashboard" class="nav-item">
                    <i class="peer-icon peer-icon-home"/>
                    <span>"Home"</span>
                </a>
                <a href="/search" class="nav-item">
                    <i class="peer-icon peer-icon-search"/>
                    <span>"Search"</span>
                </a>
                <a href="/newpost" class="nav-item add-post">
                    <i class="peer-icon peer-icon-plus"/>
                </a>
                <a href="/chat" class="nav-item active">
                    <i class="peer-icon peer-icon-chat"/>
                    <span>"Chat"</span>
                </a>
                <a href="/profile" class="nav-item">
                    <i class="peer-icon peer-icon-user"/>
                    <span>"Profile"</span>
                </a>
            </nav>
        </footer>
    }
}
