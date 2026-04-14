//! My Profile page.
//!
//! Displays the authenticated user's own profile with edit capabilities
//! and post boost features.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::profile::{get_profile, list_user_posts};
use crate::components::auth_guard::AuthGuard;
use crate::components::posts::PostCard;
use crate::components::profile::{ProfileHeader, ProfileHeaderSkeleton};
use crate::models::post::{FeedItem, PostSortType};

/// My Profile page.
///
/// Displays the current user's profile with their posts, statistics,
/// and edit/ads actions.
#[component]
pub fn MyProfilePage() -> impl IntoView {
    view! {
        <Title text="Profile - Peer Network"/>
        <AuthGuard>
            <div id="my-profile" class="site_layout profile-layout">
                <ProfilePageHeader title="Profile"/>
                <aside class="site_sidebar left-sidebar"></aside>
                <ProfileMainContent/>
                <RightSidebar/>
                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}

/// Main content area for profile page.
#[component]
fn ProfileMainContent() -> impl IntoView {
    // Fetch the current user's profile
    let profile_resource = Resource::new(
        || (),
        |_| async { get_profile(None, None).await },
    );

    // Boost post mode signal
    let boost_mode_active = RwSignal::new(false);

    let on_boost = Callback::new(move |_: ()| {
        boost_mode_active.set(true);
    });

    view! {
        <main class="site_main profile-main">
            <Suspense fallback=move || view! { <ProfileHeaderSkeleton/> }>
                {move || {
                    profile_resource.get().map(|result| {
                        match result {
                            Ok(profile) => {
                                let user_id = profile.id.clone();
                                view! {
                                    <ProfileHeader
                                        profile=profile
                                        is_own_profile=true
                                        on_boost_posts=on_boost
                                    />
                                    <ProfilePostList user_id=user_id/>
                                }.into_any()
                            }
                            Err(e) => {
                                view! {
                                    <ProfileError message=e.to_string()/>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </main>
    }
}

/// Profile post list.
#[component]
fn ProfilePostList(user_id: String) -> impl IntoView {
    let posts_resource = Resource::new(
        move || user_id.clone(),
        |user_id| async move {
            list_user_posts(
                user_id,
                vec![],
                None,
                PostSortType::Newest,
                0,
                20,
            ).await
        },
    );

    view! {
        <div class="profile-posts">
            <h3 class="posts-heading">"Posts"</h3>
            <Suspense fallback=move || view! { <PostListSkeleton/> }>
                {move || {
                    posts_resource.get().map(|result| {
                        match result {
                            Ok(response) => {
                                if response.affected_rows.is_empty() {
                                    view! {
                                        <EmptyPosts message="No posts yet"/>
                                    }.into_any()
                                } else {
                                    let items: Vec<FeedItem> = response.affected_rows
                                        .into_iter()
                                        .map(FeedItem::Post)
                                        .collect();
                                    view! {
                                        <div class="posts-grid">
                                            <For
                                                each=move || items.clone()
                                                key=|item| item.post().id.clone()
                                                let:item
                                            >
                                                <PostCard item=item/>
                                            </For>
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => {
                                view! {
                                    <ProfileError message=format!("Failed to load posts: {}", e)/>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Page header with logo and title.
#[component]
fn ProfilePageHeader(title: &'static str) -> impl IntoView {
    view! {
        <header class="site_header">
            <div class="header-content">
                <img class="logo" src="/svg/dashboard-profile.svg" alt="Peer Network"/>
                <h1>{title}</h1>
            </div>
        </header>
    }
}

/// Right sidebar with menu and widgets.
#[component]
fn RightSidebar() -> impl IntoView {
    view! {
        <aside class="site_sidebar right-sidebar">
            <nav class="sidebar-menu">
                <a href="/dashboard" class="menu-item">
                    <i class="peer-icon peer-icon-home"></i>
                    <span>"Dashboard"</span>
                </a>
                <a href="/newpost" class="menu-item">
                    <i class="peer-icon peer-icon-plus"></i>
                    <span>"New Post"</span>
                </a>
                <a href="/settings" class="menu-item">
                    <i class="peer-icon peer-icon-settings"></i>
                    <span>"Settings"</span>
                </a>
                <a href="/wallet" class="menu-item">
                    <i class="peer-icon peer-icon-wallet"></i>
                    <span>"Wallet"</span>
                </a>
            </nav>
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
                <a href="/notifications" class="nav-item">
                    <i class="peer-icon peer-icon-bell"/>
                    <span>"Alerts"</span>
                </a>
                <a href="/profile" class="nav-item active">
                    <i class="peer-icon peer-icon-user"/>
                    <span>"Profile"</span>
                </a>
            </nav>
        </footer>
    }
}

/// Error state component.
#[component]
fn ProfileError(message: String) -> impl IntoView {
    view! {
        <div class="profile-error">
            <i class="peer-icon peer-icon-alert-circle"></i>
            <h2>"Something went wrong"</h2>
            <p>{message}</p>
            <a href="/dashboard" class="button btn-blue">"Go to Dashboard"</a>
        </div>
    }
}

/// Empty posts state.
#[component]
fn EmptyPosts(message: &'static str) -> impl IntoView {
    view! {
        <div class="empty-posts">
            <i class="peer-icon peer-icon-image"></i>
            <p>{message}</p>
            <a href="/newpost" class="button btn-blue">"Create your first post"</a>
        </div>
    }
}

/// Post list skeleton loader.
#[component]
fn PostListSkeleton() -> impl IntoView {
    view! {
        <div class="post-list skeleton">
            <div class="skeleton-post"></div>
            <div class="skeleton-post"></div>
            <div class="skeleton-post"></div>
        </div>
    }
}
