//! Peer Shop page.
//!
//! Displays the @Peer_Shop account's profile and products with
//! e-commerce capabilities (price badges, checkout flow, FAQ).

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;

use crate::api::profile::{get_profile, list_user_posts, toggle_follow};
use crate::components::auth_guard::AuthGuard;
use crate::components::layout::SiteShell;
use crate::components::posts::PostCard;
use crate::components::shop::{CheckoutPopup, FaqPopup, ProductPriceBadge, ShopProfileHeader};
use crate::models::post::{FeedItem, Post, PostSortType};
use crate::models::shop::ShopProduct;

/// The Peer Shop page.
///
/// Route: `/shop`
#[component]
pub fn PeerShopPage() -> impl IntoView {
    view! {
        <AuthGuard>
            <Title text="Peer Shop - Peer Network"/>
            <SiteShell id="view-peer-shop" modifier="view-peer-shop profile-layout">
                <ShopPageHeader/>
                <aside class="site_sidebar left-sidebar">
                    <ShopFilters/>
                </aside>
                <ShopMainContent/>
                <ShopRightSidebar/>
            </SiteShell>
        </AuthGuard>
    }
}

/// Page header (top bar).
#[component]
fn ShopPageHeader() -> impl IntoView {
    view! {
        <header class="site_header">
            <div class="header-content">
                <a href="/dashboard" class="back-button" aria-label="Back to dashboard">
                    <i class="peer-icon peer-icon-arrow-left"></i>
                </a>
                <h1>"Shop"</h1>
            </div>
        </header>
    }
}

/// Left sidebar content filters.
#[component]
fn ShopFilters() -> impl IntoView {
    view! {
        <div class="shop-filters">
            <h3>"Content"</h3>
            <ul class="filter-list">
                <li class="filter-item active">"All"</li>
                <li class="filter-item">"Images"</li>
                <li class="filter-item">"Videos"</li>
                <li class="filter-item">"Audio"</li>
                <li class="filter-item">"Text"</li>
            </ul>
        </div>
    }
}

/// Main content area with shop profile header and product feed.
#[component]
fn ShopMainContent() -> impl IntoView {
    // Fetch the shop profile
    let profile_resource = Resource::new(
        || (),
        |_| async move { get_profile(Some(SHOP_ACCOUNT_USERNAME.to_string()), None).await },
    );

    // FAQ popup state
    let faq_open = RwSignal::new(false);

    // Checkout popup state
    let checkout_open = RwSignal::new(false);
    let checkout_post = RwSignal::new(None::<Post>);
    let checkout_product = RwSignal::new(None::<ShopProduct>);
    let checkout_price = RwSignal::new(0u32);

    view! {
        <main class="site_main profile-main">
            <Suspense fallback=move || view! { <ShopProfileSkeleton/> }>
                {move || {
                    profile_resource.get().map(|result| {
                        match result {
                            Ok(profile) => {
                                let user_id = profile.id.clone();
                                let is_following = RwSignal::new(profile.i_follow_this_user);
                                let profile_id = profile.id.clone();

                                let on_follow = {
                                    let pid = profile_id.clone();
                                    Callback::new(move |_: ()| {
                                        let uid = pid.clone();
                                        let currently_following = is_following.get();
                                        is_following.set(!currently_following);
                                        spawn_local(async move {
                                            let _ = toggle_follow(uid).await;
                                        });
                                    })
                                };

                                let on_info = Callback::new(move |_: ()| {
                                    faq_open.set(true);
                                });

                                view! {
                                    <ShopProfileHeader
                                        profile=profile
                                        on_info_click=on_info
                                        on_follow_click=on_follow
                                        is_following=is_following
                                    />
                                    <ShopProductFeed
                                        user_id=user_id
                                        checkout_open=checkout_open
                                        checkout_post=checkout_post
                                        checkout_product=checkout_product
                                        checkout_price=checkout_price
                                    />
                                    <FaqPopup is_open=faq_open/>
                                    <Show when=move || checkout_open.get() && checkout_post.get().is_some()>
                                        {move || {
                                            let post = checkout_post.get().unwrap();
                                            let product = checkout_product.get().unwrap_or_default();
                                            let price = checkout_price.get();
                                            view! {
                                                <CheckoutPopup
                                                    is_open=checkout_open
                                                    post=post
                                                    product=product
                                                    product_price=price
                                                />
                                            }
                                        }}
                                    </Show>
                                }.into_any()
                            }
                            Err(e) => {
                                view! {
                                    <div class="profile-error">
                                        <i class="peer-icon peer-icon-alert-circle"></i>
                                        <h2>"Something went wrong"</h2>
                                        <p>{e.to_string()}</p>
                                        <a href="/dashboard" class="button btn-blue">"Go to Dashboard"</a>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </main>
    }
}

/// Product feed grid with price badges.
#[component]
fn ShopProductFeed(
    user_id: String,
    checkout_open: RwSignal<bool>,
    checkout_post: RwSignal<Option<Post>>,
    checkout_product: RwSignal<Option<ShopProduct>>,
    checkout_price: RwSignal<u32>,
) -> impl IntoView {
    let uid = user_id.clone();
    let posts_resource = Resource::new(
        move || uid.clone(),
        |user_id| async move {
            list_user_posts(user_id, vec![], None, PostSortType::Newest, 0, 40).await
        },
    );

    view! {
        <div class="profile-posts shop-products">
            <h3 class="posts-heading">"Products"</h3>
            <Suspense fallback=move || view! { <ProductListSkeleton/> }>
                {move || {
                    posts_resource.get().map(|result| {
                        match result {
                            Ok(response) => {
                                if response.affected_rows.is_empty() {
                                    view! {
                                        <div class="empty-posts">
                                            <i class="peer-icon peer-icon-shopping-bag"></i>
                                            <p>"No products available yet."</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    let posts = response.affected_rows;
                                    view! {
                                        <div class="posts-grid shop-grid">
                                            {posts.into_iter().map(|post| {
                                                let post_for_buy = post.clone();
                                                let product_price = DEFAULT_PRODUCT_PRICE;

                                                let on_buy = {
                                                    let p = post_for_buy.clone();
                                                    move |_: leptos::ev::MouseEvent| {
                                                        checkout_post.set(Some(p.clone()));
                                                        checkout_product.set(Some(ShopProduct {
                                                            post_id: p.id.clone(),
                                                            sizes: None,
                                                            one_size_stock: None,
                                                        }));
                                                        checkout_price.set(product_price);
                                                        checkout_open.set(true);
                                                    }
                                                };

                                                let item = FeedItem::Post(post.clone());
                                                view! {
                                                    <div class="shop-card-wrapper">
                                                        <PostCard item=item/>
                                                        <div class="shop-card-overlay">
                                                            <ProductPriceBadge price=format!("{}", product_price)/>
                                                        </div>
                                                        <button
                                                            class="btn-buy btn-blue bold"
                                                            on:click=on_buy
                                                        >
                                                            "Buy"
                                                        </button>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }
                            Err(e) => {
                                view! {
                                    <div class="profile-error">
                                        <p>{format!("Failed to load products: {}", e)}</p>
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Loading skeleton for the shop profile header.
#[component]
fn ShopProfileSkeleton() -> impl IntoView {
    view! {
        <div class="shop-profile-header skeleton">
            <div class="shop-header-gradient">
                <div class="shop-header-content">
                    <div class="shop-avatar-wrapper skeleton-avatar"></div>
                    <div class="shop-info">
                        <div class="skeleton-text skeleton-title"></div>
                        <div class="skeleton-text skeleton-slug"></div>
                        <div class="skeleton-text skeleton-bio"></div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Loading skeleton for the product list.
#[component]
fn ProductListSkeleton() -> impl IntoView {
    view! {
        <div class="posts-grid shop-grid">
            {(0..6).map(|_| view! {
                <div class="card skeleton">
                    <div class="skeleton-media"></div>
                    <div class="skeleton-text"></div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

/// Right sidebar.
#[component]
fn ShopRightSidebar() -> impl IntoView {
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
                <a href="/wallet" class="menu-item">
                    <i class="peer-icon peer-icon-wallet"></i>
                    <span>"Wallet"</span>
                </a>
            </nav>
        </aside>
    }
}

/// Shop account username (used to look up the shop profile by slug/username).
/// The legacy app uses the `@Peer_Shop` system account.
const SHOP_ACCOUNT_USERNAME: &str = "Peer_Shop";

/// Default product price when Firebase data is not available.
const DEFAULT_PRODUCT_PRICE: u32 = 500;
