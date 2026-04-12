//! View Profile page.
//!
//! Displays another user's profile with follow/block/report actions.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::api::profile::{get_profile, list_user_posts};
use crate::components::auth_guard::AuthGuard;
use crate::components::posts::PostCard;
use crate::components::profile::{ProfileHeader, ProfileHeaderSkeleton};
use crate::models::post::{FeedItem, PostSortType};

/// View another user's profile page.
///
/// Route: `/profile/:slug` or `/u/:slug`
#[component]
pub fn ViewProfilePage() -> impl IntoView {
    let params = use_params_map();

    // Extract user slug from route params
    let user_slug = move || {
        params.get()
            .get("slug")
            .map(|s| s.to_string())
    };

    view! {
        <AuthGuard>
            <div id="view-profile" class="site_layout profile-layout">
                <ViewProfileHeader/>
                <aside class="site_sidebar left-sidebar"></aside>
                <ViewProfileMainContent user_slug/>
                <RightSidebar/>
                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}

/// Main content area for viewing another user's profile.
#[component]
fn ViewProfileMainContent(
    user_slug: impl Fn() -> Option<String> + Send + Sync + 'static,
) -> impl IntoView {
    // Fetch the profile based on the slug
    let profile_resource = Resource::new(
        user_slug,
        |slug| async move {
            match slug {
                Some(slug) => get_profile(Some(slug), None).await,
                None => Err(ServerFnError::new("No user specified")),
            }
        },
    );

    view! {
        <main class="site_main profile-main">
            <Suspense fallback=move || view! { <ProfileHeaderSkeleton/> }>
                {move || {
                    profile_resource.get().map(|result| {
                        match result {
                            Ok(profile) => {
                                let page_title = format!("{} - Peer Network", profile.username);
                                let user_id = profile.id.clone();
                                
                                view! {
                                    <Title text=page_title/>
                                    <ProfileHeader
                                        profile=profile
                                        is_own_profile=false
                                    />
                                    <UserPostList user_id=user_id/>
                                }.into_any()
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                if error_msg.contains("not found") || error_msg.contains("Profile not found") {
                                    view! {
                                        <Title text="Profile Not Found - Peer Network"/>
                                        <ProfileNotFound/>
                                    }.into_any()
                                } else {
                                    view! {
                                        <Title text="Error - Peer Network"/>
                                        <ProfileError message=error_msg/>
                                    }.into_any()
                                }
                            }
                        }
                    })
                }}
            </Suspense>
        </main>
    }
}

/// User's post list.
#[component]
fn UserPostList(user_id: String) -> impl IntoView {
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
                                        <EmptyPosts/>
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

/// Page header for view profile.
#[component]
fn ViewProfileHeader() -> impl IntoView {
    view! {
        <header class="site_header">
            <div class="header-content">
                <a href="/dashboard" class="back-button" aria-label="Back to dashboard">
                    <i class="peer-icon peer-icon-arrow-left"></i>
                </a>
                <h1>"Profile"</h1>
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
                <a href="/profile" class="menu-item">
                    <i class="peer-icon peer-icon-user"></i>
                    <span>"My Profile"</span>
                </a>
                <a href="/newpost" class="menu-item">
                    <i class="peer-icon peer-icon-plus"></i>
                    <span>"New Post"</span>
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
                <a href="/profile" class="nav-item">
                    <i class="peer-icon peer-icon-user"/>
                    <span>"Profile"</span>
                </a>
            </nav>
        </footer>
    }
}

/// Profile not found state.
#[component]
fn ProfileNotFound() -> impl IntoView {
    view! {
        <div class="profile-not-found">
            <i class="peer-icon peer-icon-user-x"></i>
            <h2>"Profile Not Found"</h2>
            <p>"The user you're looking for doesn't exist or has been removed."</p>
            <a href="/dashboard" class="button btn-blue">"Go to Dashboard"</a>
        </div>
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

/// Empty posts state for other users.
#[component]
fn EmptyPosts() -> impl IntoView {
    view! {
        <div class="empty-posts">
            <i class="peer-icon peer-icon-image"></i>
            <p>"This user hasn't posted anything yet."</p>
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
