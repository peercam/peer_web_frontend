//! My Profile page.
//!
//! Displays the authenticated user's own profile with edit capabilities
//! and post boost features.

use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::api::profile::{get_profile, list_user_posts};
use crate::components::auth_guard::AuthGuard;
use crate::components::filters::{ContentFilter, SortFilter};
use crate::components::layout::SiteShell;
use crate::components::posts::PostCard;
use crate::components::profile::{ProfileHeader, ProfileHeaderSkeleton};
use crate::hooks::use_infinite_scroll;
use crate::models::post::FeedItem;
use crate::state::filters::{provide_filter_context, use_filter_state};

/// My Profile page.
///
/// Displays the current user's profile with their posts, statistics,
/// and edit/ads actions.
#[component]
pub fn MyProfilePage() -> impl IntoView {
    provide_filter_context();

    view! {
        <Title text="Profile - Peer Network"/>
        <AuthGuard>
            <QueryParamRedirect/>
            <SiteShell id="my-profile" modifier="profile-layout">
                <ProfilePageHeader title="Profile"/>
                <ProfileFilterSidebar/>
                <ProfileMainContent/>
                <RightSidebar/>
            </SiteShell>
        </AuthGuard>
    }
}

/// Main content area for profile page.
#[component]
fn ProfileMainContent() -> impl IntoView {
    // Fetch the current user's profile
    let profile_resource = Resource::new(|| (), |_| async { get_profile(None, None).await });

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

/// Number of posts to load per batch.
const POSTS_PER_PAGE: i32 = 20;

/// Profile post list with infinite scroll.
#[component]
fn ProfilePostList(user_id: String) -> impl IntoView {
    let filters = use_filter_state();

    let feed = RwSignal::new(Vec::<FeedItem>::new());
    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);

    // Reset feed when filters change
    Effect::new(move |prev_version: Option<u32>| {
        let current_version = filters.version.get();
        if prev_version.is_some() && prev_version != Some(current_version) {
            feed.set(Vec::new());
            offset.set(0);
            has_more.set(true);
        }
        current_version
    });

    let uid = user_id.clone();
    let load_posts = Rc::new(move || {
        if is_loading.get() || !has_more.get() {
            return;
        }
        is_loading.set(true);

        let user_id = uid.clone();
        let current_offset = offset.get();
        let filter_by = filters.get_filter_by();
        let sort_by = filters.sort_by.get();

        spawn_local(async move {
            match list_user_posts(
                user_id,
                filter_by,
                None,
                sort_by,
                current_offset,
                POSTS_PER_PAGE,
            )
            .await
            {
                Ok(response) => {
                    let new_posts = response.affected_rows;
                    let has_new = !new_posts.is_empty();
                    if has_new {
                        feed.update(|f| {
                            for post in new_posts {
                                f.push(FeedItem::Post(post));
                            }
                        });
                        offset.update(|o| *o += POSTS_PER_PAGE);
                    }
                    has_more.set(has_new && response.counter > current_offset + POSTS_PER_PAGE);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to load posts: {:?}", e);
                }
            }
            is_loading.set(false);
        });
    });

    let scroll = use_infinite_scroll(is_loading, has_more, {
        let load_posts = load_posts.clone();
        move || load_posts()
    });

    // Initial load
    {
        let load_posts = load_posts.clone();
        Effect::new(move |_| {
            if feed.get().is_empty() && !is_loading.get() {
                load_posts();
            }
        });
    }

    view! {
        <div class="profile-posts">
            <h3 class="posts-heading">"Posts"</h3>
            <div class="posts-grid">
                <For
                    each=move || feed.get()
                    key=|item| item.post().id.clone()
                    children=move |item| {
                        view! { <PostCard item=item/> }
                    }
                />
            </div>

            <div class="post-loader" node_ref=scroll.loader_ref>
                <Show when=move || is_loading.get()>
                    <div class="loading-indicator">
                        <img src="/svg/logo_farbe.svg" alt="Loading..." class="loading-spinner"/>
                    </div>
                </Show>
            </div>

            <Show when=move || feed.get().is_empty() && !is_loading.get() && !has_more.get()>
                <EmptyPosts message="No posts yet"/>
            </Show>
        </div>
    }
}

/// Redirect legacy `?user=<uuid>` query param to `/u/<uuid>`.
#[component]
#[allow(clippy::unused_unit)]
fn QueryParamRedirect() -> impl IntoView {
    let params = use_query_map();

    Effect::new(move |_| {
        if let Some(user_id) = params.get().get("user")
            && !user_id.is_empty()
        {
            let navigate = use_navigate();
            navigate(&format!("/u/{}", user_id), Default::default());
        }
    });

    view! {}
}

/// Profile filter sidebar (content type + sort only).
#[component]
fn ProfileFilterSidebar() -> impl IntoView {
    let filters = use_filter_state();

    view! {
        <aside
            class="site_sidebar left-sidebar left-sidebar-profile"
            class:collapsed=move || filters.is_collapsed.get()
        >
            <div class="inner-scroll for-filters">
                <div class="inner-scroll-filters">
                    <ContentFilter/>
                    <SortFilter/>
                </div>
            </div>
        </aside>
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
