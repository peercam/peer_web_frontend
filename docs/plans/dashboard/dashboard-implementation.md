# Dashboard Implementation Plan

**Feature:** Dashboard  
**Priority:** #3 (after Login/Auth)  
**Status:** � Mostly Implemented (minor gaps)  
**Created:** 2026-04-10  
**Updated:** 2026-04-14

---

## Overview

Implement the main dashboard page for the Leptos frontend — the core user experience after login. The dashboard displays a post feed with infinite scroll, filtering, sorting, and real-time user interactions.

### Goals

1. Full parity with legacy `dashboard.php` user experience
2. Infinite scroll post feed with lazy loading
3. Content type filters (image, video, text, audio)
4. Feed filters (all, followers, following)
5. Sort options (newest, trending, most liked, etc.)
6. Post interactions (like, dislike, save, view, report)
7. User search functionality
8. Responsive layout with profile widget and navigation

---

## Scope

### In Scope

- [x] Dashboard page UI (`/dashboard`)
- [x] Post list component with infinite scroll
- [x] Post card component (image, video, audio, text variants)
- [x] Content type filter sidebar
- [x] Feed filter (all/followers/following)
- [x] Sort options (newest, trending, likes, views, comments)
- [x] Title/tag search bar
- [x] User search with dropdown
- [x] Profile widget (sidebar) — ⚠️ placeholder only, not wired to auth context
- [x] Main navigation menu
- [x] Quick actions (new post link)
- [ ] Post click → view post modal/overlay — ⚠️ click handler is a TODO stub
- [x] Like/dislike/save interactions
- [x] View tracking (mark posts as viewed)
- [x] Advertisement post integration
- [x] Filter persistence (localStorage)
- [x] Loading states and skeletons
- [x] Empty state ("No posts found")

### Out of Scope (Future Work)

- Full view post page (separate feature)
- New post creation (separate feature)
- Comments (separate feature, but UI placeholder included)
- Chat widget
- Wallet widget
- Audio/video playback (basic placeholder)

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `dashboard.php` | Page template with layout structure |
| `js/dashboard.js` | Infinite scroll observer, user search, filter toggles |
| `js/load_posts.js` | Posts API calls, post card rendering |
| `js/posts.js` | Post actions (like, dislike, view), GraphQL queries |
| `js/global.js` | Auth cookies, user info, utilities |
| `css/dashboard.css` | Dashboard-specific styles |
| `css/all-post.css` | Post card grid styles |
| `css/view-post.css` | View post overlay styles |
| `template-parts/sidebars/widget-*.php` | Sidebar widgets |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  HEADER (fixed): Logo + "Dashboard" title                  │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│  (16rem)     │  - Search bar           │  (17rem)          │
│              │  - Post grid            │                   │
│  - Filters   │  - Infinite scroll      │  - Profile widget │
│    (collapse)│  - Loading indicator    │  - Main menu      │
│  - Content   │                         │  - New post btn   │
│  - Feed      │                         │  - Version        │
│  - Sort      │                         │                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### Key Features

1. **Infinite Scroll**
   - IntersectionObserver on `#post_loader` element
   - Loads 20 posts per batch
   - Merges advertisement posts with regular posts
   - Offset tracking for pagination

2. **Content Filters (Checkboxes)**
   - IMAGE, VIDEO, TEXT, AUDIO
   - Multiple can be selected
   - Stored in localStorage

3. **Feed Filters (Radio)**
   - All (default)
   - FOLLOWER (posts from followers)
   - FOLLOWED (posts from users you follow)

4. **Sort Options (Radio)**
   - NEWEST (default)
   - TRENDING
   - LIKES
   - VIEWS
   - COMMENTS
   - DISLIKES

5. **Search**
   - Title search (debounced input)
   - Tag search (hashtag filter)
   - User search (dropdown with results)

6. **Filter Persistence**
   - `selectedContentTypes` → JSON array in localStorage
   - `selected-feed` → localStorage
   - `isFiltersCollapsed` → localStorage

---

## Backend API Reference

### `listPosts` Query

```graphql
query ListPosts(
  $filterBy: [PostFilterType!]
  $contentFilterBy: ContentFilterType
  $IgnorList: IgnoreOption
  $sortBy: PostSortType
  $userid: ID
  $title: String
  $tag: String
  $from: Date
  $to: Date
  $offset: Int
  $limit: Int
) {
  listPosts(
    filterBy: $filterBy
    contentFilterBy: $contentFilterBy
    IgnorList: $IgnorList
    sortBy: $sortBy
    userid: $userid
    title: $title
    tag: $tag
    from: $from
    to: $to
    offset: $offset
    limit: $limit
  ) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      id
      contenttype
      title
      media
      cover
      mediadescription
      createdat
      amountlikes
      amountviews
      amountcomments
      amountdislikes
      amounttrending
      isliked
      isviewed
      isdisliked
      issaved
      isreported
      tags
      hasActiveReports
      visibilityStatus
      isHiddenForUsers
      user {
        id
        username
        slug
        img
        isfollowed
        isfollowing
        isfriend
      }
    }
  }
}
```

#### Enums

```graphql
enum PostFilterType {
  IMAGE
  AUDIO
  VIDEO
  TEXT
  FOLLOWED
  FOLLOWER
  VIEWED
  FRIENDS
}

enum PostSortType {
  NEWEST
  TRENDING
  LIKES
  DISLIKES
  VIEWS
  COMMENTS
  FOR_ME
  OLDEST
  FOLLOWER
  FOLLOWED
  RELEVANT
  FRIENDS
}

enum ContentFilterType {
  MYGRANDMALIKES   # Stricter filtering
  MYGRANDMAHATES   # Relaxed filtering
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11501` | Posts retrieved successfully |
| `21518` | No posts found |
| `30103` | Invalid filterBy or sortBy |
| `30201` | Invalid user UUID |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `30209` | Invalid post UUID |
| `30210` | Invalid title length |
| `30211` | Invalid tag format |
| `30212` | Invalid from date |
| `30213` | Invalid to date |
| `31510` | Post not found (by ID) |
| `60501` | Not authenticated |

### `listAdvertisementPosts` Query

```graphql
query ListAdvertisementPosts(
  $offset: Int
  $limit: Int
  $contentFilterBy: ContentFilterType
  $userid: ID
  $title: String
  $tag: String
) {
  listAdvertisementPosts(
    offset: $offset
    limit: $limit
    contentFilterBy: $contentFilterBy
    userid: $userid
    title: $title
    tag: $tag
  ) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      post { ... }
      advertisement {
        advertisementid
        advertisementtype
        startdate
        enddate
      }
    }
  }
}
```

### `searchUser` Query

```graphql
query SearchUser($username: String!, $offset: Int, $limit: Int) {
  searchUser(username: $username, offset: $offset, limit: $limit) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      id
      username
      slug
      img
      biography
    }
  }
}
```

### `resolvePostAction` Mutation

```graphql
mutation ResolvePostAction($postid: ID!, $action: PostActionType!) {
  resolvePostAction(postid: $postid, action: $action) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
  }
}

enum PostActionType {
  VIEW
  LIKE
  DISLIKE
  SAVE
  REPORT
  UNLIKE
  UNDISLIKE
  UNSAVE
}
```

### `getUserInfo` Query

```graphql
query GetUser($id: ID!) {
  getUser(id: $id) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    affectedRows {
      id
      username
      slug
      img
      biography
      amountFollowers
      amountFollowing
      amountPeers
      userPreferences {
        contentFilteringSeverityLevel
      }
    }
  }
}
```

---

## Implementation Plan

### Phase 1: Core Infrastructure & Models

#### 1.1 Post Models (`src/models/post.rs`)

```rust
/// Content type of a post.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentType {
    Image,
    Video,
    Audio,
    Text,
}

/// Post filter types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PostFilterType {
    Image,
    Video,
    Audio,
    Text,
    Followed,
    Follower,
    Friends,
}

/// Post sort options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum PostSortType {
    #[default]
    Newest,
    Trending,
    Likes,
    Dislikes,
    Views,
    Comments,
    ForMe,
}

/// Content filtering severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentFilterType {
    /// Stricter: hides flagged content.
    Mygrandmalikes,
    /// Relaxed: shows more content.
    Mygrandmahates,
}

/// Block list filtering option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IgnoreOption {
    Yes,
    No,
}

/// Simplified user embedded in posts.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostUser {
    pub id: String,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    pub isfollowed: bool,
    pub isfollowing: bool,
    pub isfriend: bool,
}

/// Post data from API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: String,
    pub contenttype: ContentType,
    pub title: String,
    pub media: Option<String>,
    pub cover: Option<String>,
    pub mediadescription: Option<String>,
    pub createdat: String,
    pub amountlikes: i32,
    pub amountviews: i32,
    pub amountcomments: i32,
    pub amountdislikes: i32,
    pub amounttrending: Option<i32>,
    pub isliked: bool,
    pub isviewed: bool,
    pub isdisliked: bool,
    pub issaved: bool,
    pub isreported: bool,
    pub tags: Option<Vec<String>>,
    pub user: PostUser,
}

/// Advertisement metadata attached to promoted posts.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisementInfo {
    pub advertisementid: String,
    pub advertisementtype: String,
    pub startdate: String,
    pub enddate: String,
}

/// Advertisement post wrapper (from `listAdvertisementPosts`).
///
/// Note: This is a separate response type from regular posts.
/// When merging ads into the feed, convert to `Post` with an `is_ad` marker
/// or use an enum like `FeedItem { Regular(Post), Ad(AdvertisementPost) }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisementPost {
    pub post: Post,
    pub advertisement: AdvertisementInfo,
}

/// Advertisement list response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Vec<AdvertisementPost>,
}

/// Unified feed item for rendering (regular post or ad).
#[derive(Debug, Clone)]
pub enum FeedItem {
    Post(Post),
    Ad { post: Post, ad_info: AdvertisementInfo },
}

impl FeedItem {
    pub fn post(&self) -> &Post {
        match self {
            FeedItem::Post(p) => p,
            FeedItem::Ad { post, .. } => post,
        }
    }
    
    pub fn is_ad(&self) -> bool {
        matches!(self, FeedItem::Ad { .. })
    }
}

/// List posts response.
///
/// Matches the `PostListResponse` GraphQL type. The `meta` field
/// contains standard response info (status, codes) while `affectedRows`
/// holds the actual posts.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Vec<Post>,
}

impl PostListResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}
```

#### 1.2 User Info Model (`src/models/user.rs`)

```rust
/// Logged-in user profile info.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    pub biography: Option<String>,
    pub amount_followers: i32,
    pub amount_following: i32,
    pub amount_peers: i32,
}
```

#### 1.3 Filter State (`src/state/filters.rs`)

```rust
use leptos::prelude::*;

/// Dashboard filter state.
#[derive(Clone)]
pub struct FilterState {
    /// Content types to show (empty = all).
    pub content_types: RwSignal<Vec<PostFilterType>>,
    /// Feed filter (None = all).
    pub feed_filter: RwSignal<Option<PostFilterType>>,
    /// Sort order.
    pub sort_by: RwSignal<PostSortType>,
    /// Title search query.
    pub title_query: RwSignal<String>,
    /// Tag filter.
    pub tag_query: RwSignal<String>,
    /// Sidebar collapsed state.
    pub is_collapsed: RwSignal<bool>,
}

impl FilterState {
    /// Load filter state from localStorage.
    pub fn from_storage() -> Self;
    
    /// Persist current filter state to localStorage.
    pub fn save_to_storage(&self);
}
```

### Phase 2: Posts API Layer

#### 2.1 Posts Server Functions (`src/api/posts.rs`)

```rust
/// Fetch paginated posts with filters.
#[server(ListPosts, "/api")]
pub async fn list_posts(
    filter_by: Vec<PostFilterType>,
    content_filter_by: Option<ContentFilterType>,
    sort_by: PostSortType,
    title: Option<String>,
    tag: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<PostListResponse, ServerFnError>;

/// Fetch advertisement posts.
#[server(ListAdPosts, "/api")]
pub async fn list_ad_posts(
    content_filter_by: Option<ContentFilterType>,
    offset: i32,
    limit: i32,
    title: Option<String>,
    tag: Option<String>,
) -> Result<AdListResponse, ServerFnError>;

/// Perform action on post (like, dislike, save, view).
#[server(PostAction, "/api")]
pub async fn post_action(
    post_id: String,
    action: PostActionType,
) -> Result<(), ServerFnError>;

/// Search users by username.
#[server(SearchUsers, "/api")]
pub async fn search_users(
    username: String,
    limit: i32,
) -> Result<Vec<UserSearchResult>, ServerFnError>;
```

#### 2.2 GraphQL Queries (`src/api/graphql.rs` additions)

```rust
pub const LIST_POSTS_QUERY: &str = r#"
    query ListPosts(
        $filterBy: [PostFilterType!],
        $contentFilterBy: ContentFilterType,
        $sortBy: PostSortType,
        $title: String,
        $tag: String,
        $offset: Int,
        $limit: Int
    ) {
        listPosts(
            filterBy: $filterBy,
            contentFilterBy: $contentFilterBy,
            sortBy: $sortBy,
            title: $title,
            tag: $tag,
            offset: $offset,
            limit: $limit
        ) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            counter
            affectedRows {
                id
                contenttype
                title
                media
                cover
                mediadescription
                createdat
                amountlikes
                amountviews
                amountcomments
                amountdislikes
                isliked
                isviewed
                isdisliked
                issaved
                tags
                user {
                    id
                    username
                    slug
                    img
                    isfollowed
                    isfollowing
                    isfriend
                }
            }
        }
    }
"#;

pub const POST_ACTION_MUTATION: &str = r#"
    mutation ResolvePostAction($postid: ID!, $action: PostActionType!) {
        resolvePostAction(postid: $postid, action: $action) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
        }
    }
"#;

pub const SEARCH_USERS_QUERY: &str = r#"
    query SearchUser($username: String!, $offset: Int, $limit: Int) {
        searchUser(username: $username, offset: $offset, limit: $limit) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            counter
            affectedRows {
                id
                username
                slug
                img
            }
        }
    }
"#;
```

### Phase 3: Dashboard Layout Components

#### 3.1 Dashboard Page (`src/pages/dashboard.rs`)

```rust
#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <Title text="Peer Network - Dashboard"/>
        <AuthGuard>
            <div id="dashboard" class="site_layout">
                <DashboardHeader/>
                <LeftSidebar/>
                <MainContent/>
                <RightSidebar/>
                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}
```

#### 3.2 Dashboard Header (`src/components/dashboard/header.rs`)

```rust
#[component]
pub fn DashboardHeader() -> impl IntoView {
    view! {
        <header class="site-header header-dashboard">
            <img class="logo" src="/svg/Home.svg" alt="Peer Network"/>
            <h1 class="dashboard_h1">"Dashboard"</h1>
        </header>
    }
}
```

#### 3.3 Left Sidebar (`src/components/dashboard/left_sidebar.rs`)

```rust
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let is_collapsed = use_filter_state().is_collapsed;
    
    view! {
        <aside class="left-sidebar left-sidebar-dashboard" class:collapsed=is_collapsed>
            <div class="inner-scroll for-filters">
                <div class="inner-scroll-filters">
                    <ContentFilter/>
                    <FeedFilter/>
                    <SortFilter/>
                </div>
                <CollapseButton/>
            </div>
        </aside>
    }
}
```

#### 3.4 Right Sidebar (`src/components/dashboard/right_sidebar.rs`)

```rust
#[component]
pub fn RightSidebar() -> impl IntoView {
    view! {
        <aside class="right-sidebar right-sidebar-dashboard">
            <div class="inner-scroll">
                <ProfileWidget/>
                <MainMenu/>
                <AddPostButton/>
                <VersionWidget/>
            </div>
        </aside>
    }
}
```

### Phase 4: Post Feed Components

#### 4.1 Post List (`src/components/posts/post_list.rs`)

```rust
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

#[component]
pub fn PostList() -> impl IntoView {
    let posts = RwSignal::new(Vec::<Post>::new());
    let offset = RwSignal::new(0);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    
    // Intersection observer for infinite scroll
    let loader_ref = NodeRef::<html::Div>::new();
    
    // Load more posts when loader becomes visible
    Effect::new(move || {
        let Some(el) = loader_ref.get() else { return };
        
        // Create observer callback
        let callback = Closure::<dyn Fn(Vec<IntersectionObserverEntry>)>::new(
            move |entries: Vec<IntersectionObserverEntry>| {
                if let Some(entry) = entries.first() {
                    if entry.is_intersecting() && !is_loading.get() && has_more.get() {
                        load_more_posts(posts, offset, is_loading, has_more);
                    }
                }
            }
        );
        
        let mut options = IntersectionObserverInit::new();
        options.root_margin("0px 0px 100% 0px");
        options.threshold(&JsValue::from_f64(0.1));
        
        let observer = IntersectionObserver::new_with_options(
            callback.as_ref().unchecked_ref(),
            &options,
        ).expect("Failed to create IntersectionObserver");
        
        observer.observe(&el);
        
        // Cleanup: disconnect observer when component unmounts
        on_cleanup(move || {
            observer.disconnect();
            drop(callback); // prevent callback from being dropped early
        });
    });
    
    view! {
        <div id="allpost" class="list_all_post">
            <For
                each=move || posts.get()
                key=|post| post.id.clone()
                children=|post| view! { <PostCard post=post/> }
            />
        </div>
        
        <div id="post_loader" node_ref=loader_ref>
            <Show when=move || is_loading.get()>
                <img src="/svg/logo_farbe.svg" alt="loading"/>
            </Show>
        </div>
        
        <Show when=move || posts.get().is_empty() && !is_loading.get()>
            <div class="no_post_found">"No Post found..."</div>
        </Show>
    }
}
```

#### 4.2 Post Card (`src/components/posts/post_card.rs`)

```rust
#[component]
pub fn PostCard(post: Post) -> impl IntoView {
    let is_liked = RwSignal::new(post.isliked);
    let like_count = RwSignal::new(post.amountlikes);
    
    let on_click = move |_| {
        // Open view post modal/overlay
    };
    
    let on_like = move |ev: MouseEvent| {
        ev.stop_propagation();
        toggle_like(post.id.clone(), is_liked, like_count);
    };

    view! {
        <section
            class="card"
            class:card-shop-product=post.is_ad
            tabindex="0"
            content=post.contenttype.to_string()
            on:click=on_click
        >
            <div class="post">
                <div class="shadow"/>
                <div class="post-inhalt">
                    <PostCardHeader user=post.user.clone()/>
                    <PostMedia post=post.clone()/>
                    <PostContent post=post.clone()/>
                    <PostActions
                        post_id=post.id.clone()
                        is_liked=is_liked
                        like_count=like_count
                        view_count=post.amountviews
                        comment_count=post.amountcomments
                    />
                </div>
            </div>
        </section>
    }
}
```

#### 4.3 Post Card Variants

- `PostCardImage` — image posts with cover
- `PostCardVideo` — video posts with play button overlay
- `PostCardAudio` — audio posts with player graphic
- `PostCardText` — text-only posts

### Phase 5: Filter Components

#### 5.1 Content Filter (`src/components/filters/content_filter.rs`)

```rust
#[component]
pub fn ContentFilter() -> impl IntoView {
    let filters = use_filter_state();
    let is_expanded = RwSignal::new(false);
    
    let content_types = [
        (PostFilterType::Image, "Photo", "/svg/photo.svg"),
        (PostFilterType::Video, "Video", "/svg/videos.svg"),
        (PostFilterType::Text, "Text", "/svg/text.svg"),
        (PostFilterType::Audio, "Music", "/svg/music.svg"),
    ];
    
    view! {
        <section class="filter-section">
            <button
                type="button"
                class="filter-toggle filter-section-header"
                aria-expanded=move || is_expanded.get()
                aria-controls="content-options"
                on:click=move |_| is_expanded.update(|v| *v = !*v)
            >
                <img src="/svg/content-icon.svg" class="section-icon"/>
                <div class="filter-section-container">
                    <span class="section-title">"Content"</span>
                    <img src="/svg/content-arrow.svg" class="section-arrow"/>
                </div>
            </button>
            
            <div class="filter-options" class:open=is_expanded>
                <div class="filterGroup">
                    <For
                        each=move || content_types.iter().cloned()
                        key=|(t, _, _)| *t
                        children=move |(filter_type, label, icon)| {
                            // Leptos 0.8: use Memo::new instead of create_memo
                            let is_checked = Memo::new(move |_| {
                                filters.content_types.get().contains(&filter_type)
                            });
                            
                            view! {
                                <label class="filterButton">
                                    <input
                                        type="checkbox"
                                        checked=is_checked
                                        on:change=move |_| toggle_content_filter(filter_type)
                                    />
                                    <img src=icon/>
                                    <span>{label}</span>
                                </label>
                            }
                        }
                    />
                </div>
            </div>
        </section>
    }
}
```

#### 5.2 Feed Filter (`src/components/filters/feed_filter.rs`)

Radio buttons for All / Followers / Following.

#### 5.3 Sort Filter (`src/components/filters/sort_filter.rs`)

Radio buttons for sort options.

### Phase 6: Search Components

#### 6.1 Search Bar (`src/components/search/search_bar.rs`)

```rust
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::closure::Closure;

/// Utility: create a debounced effect that only runs after `delay` of inactivity.
fn use_debounce<T: Clone + 'static>(
    source: impl Fn() -> T + 'static,
    delay_ms: u32,
    callback: impl Fn(T) + 'static,
) {
    let timeout_handle = StoredValue::new(None::<i32>);
    
    Effect::new(move || {
        let value = source();
        
        // Clear previous timeout
        if let Some(handle) = timeout_handle.get_value() {
            web_sys::window()
                .unwrap()
                .clear_timeout_with_handle(handle);
        }
        
        // Set new timeout
        let callback = callback.clone();
        let closure = Closure::once(Box::new(move || callback(value)) as Box<dyn FnOnce()>);
        let handle = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay_ms as i32,
            )
            .unwrap();
        closure.forget();
        timeout_handle.set_value(Some(handle));
    });
}

#[component]
pub fn SearchBar() -> impl IntoView {
    let filters = use_filter_state();
    let title_input = RwSignal::new(String::new());
    
    // Debounced search (300ms delay)
    use_debounce(
        move || title_input.get(),
        300,
        move |query| filters.title_query.set(query),
    );
    
    view! {
        <div class="search-container">
            <input
                type="text"
                id="searchTitle"
                placeholder="Search posts..."
                prop:value=title_input
                on:input=move |ev| title_input.set(event_target_value(&ev))
            />
            <img src="/svg/lupe.svg" class="lupe"/>
        </div>
    }
}
```

#### 6.2 User Search (`src/components/search/user_search.rs`)

```rust
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;

#[component]
pub fn UserSearch() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<UserSearchResult>::new());
    let is_open = RwSignal::new(false);
    let timeout_handle = StoredValue::new(None::<i32>);
    
    // Debounced user search (300ms delay, min 3 chars)
    Effect::new(move || {
        let q = query.get();
        
        // Clear previous timeout
        if let Some(handle) = timeout_handle.get_value() {
            web_sys::window().unwrap().clear_timeout_with_handle(handle);
        }
        
        if q.len() < 3 {
            results.set(vec![]);
            is_open.set(false);
            return;
        }
        
        // Debounce: wait 300ms before searching
        let closure = Closure::once(Box::new(move || {
            spawn_local(async move {
                if let Ok(users) = search_users(q, 20).await {
                    results.set(users);
                    is_open.set(true);
                }
            });
        }) as Box<dyn FnOnce()>);
        
        let handle = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                300,
            )
            .unwrap();
        closure.forget();
        timeout_handle.set_value(Some(handle));
    });
    
    view! {
        <div class="search-box" class:active=is_open>
            <input
                type="text"
                id="searchUser"
                placeholder="@username"
                prop:value=query
                on:input=move |ev| query.set(event_target_value(&ev))
            />
            
            <div 
                id="userDropdown" 
                class="dropdown" 
                class:active=is_open
                // Use mousedown instead of click to fire before blur
                on:mousedown=|ev| ev.prevent_default()
            >
                <For
                    each=move || results.get()
                    key=|user| user.id.clone()
                    children=|user| view! {
                        <UserDropdownItem user=user on_select=move |_| is_open.set(false)/>
                    }
                />
            </div>
        </div>
    }
}
```

### Phase 7: Sidebar Widgets

#### 7.1 Profile Widget (`src/components/widgets/profile_widget.rs`)

```rust
#[component]
pub fn ProfileWidget() -> impl IntoView {
    let user_info = use_user_info(); // From auth context
    
    view! {
        <div class="widget widget-margin-bottom">
            <A href="/profile" class="widget-inner widget-type-box widget-profile">
                <div class="profile-header">
                    <div class="cropContainer">
                        <span class="online_status"/>
                        <img
                            src=move || user_info.get().map(|u| u.img).flatten()
                                .unwrap_or_else(|| "/svg/noname.svg".to_string())
                            alt="Profile Picture"
                            class="profilbild profile-picture"
                        />
                    </div>
                    <div class="pro-name">
                        <div class="username">{move || user_info.get().map(|u| u.username)}</div>
                        <p class="slug">{move || user_info.get().map(|u| format!("#{}", u.slug))}</p>
                    </div>
                </div>
                
                <div class="stats">
                    <Stat label="Followers" value=move || user_info.get().map(|u| u.amount_followers)/>
                    <Stat label="Peers" value=move || user_info.get().map(|u| u.amount_peers)/>
                    <Stat label="Following" value=move || user_info.get().map(|u| u.amount_following)/>
                </div>
            </A>
        </div>
    }
}
```

#### 7.2 Main Menu (`src/components/widgets/main_menu.rs`)

```rust
#[component]
pub fn MainMenu() -> impl IntoView {
    let location = use_location();
    let current_path = move || location.pathname.get();
    
    let menu_items = [
        ("/dashboard", "Dashboard", "peer-icon-home-alt", "peer-icon-home"),
        ("/wallet", "Wallet", "peer-icon-wallet", "peer-icon-wallet-filled"),
        ("/shop", "Shop", "peer-icon-shop", "peer-icon-shop"),
        ("/settings", "Settings", "peer-icon-setting", "peer-icon-setting-filled"),
    ];
    
    view! {
        <div class="widget widget-margin-bottom">
            <div class="widget-inner widget-type-box widget-main-menu">
                <ul class="menu">
                    <For
                        each=move || menu_items.iter().cloned()
                        key=|(path, _, _, _)| *path
                        children=move |(path, label, icon, icon_filled)| {
                            // Leptos 0.8: use Memo::new instead of create_memo
                            let is_active = Memo::new(move |_| current_path() == path);
                            
                            view! {
                                <li class="menu-item" class:active=is_active>
                                    <A href=path>
                                        <i class=format!("peer-icon {}", icon)/>
                                        <i class=format!("filled peer-icon {}", icon_filled)/>
                                        {label}
                                    </A>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
        </div>
    }
}
```

---

## Files to Create/Modify

### Structural Notes

> **Component organization change:** Currently, components live flat in `src/components/`
> (e.g., `auth_guard.rs`, `toast.rs`). This plan introduces nested submodules
> (`dashboard/`, `posts/`, `filters/`, `search/`, `widgets/`). Update `src/components/mod.rs`
> accordingly to re-export the new modules.

> **User model:** `src/models/user.rs` already exists with registration types.
> Add `UserInfo` struct to the existing file rather than creating a new one.

### New Files

| File | Purpose |
|------|---------|
| `src/models/post.rs` | Post and filter types |
| `src/api/posts.rs` | Post-related server functions |
| `src/state/filters.rs` | Filter state management |
| `src/pages/dashboard.rs` | Dashboard page component |
| `src/components/dashboard/mod.rs` | Dashboard components module |
| `src/components/dashboard/header.rs` | Dashboard header |
| `src/components/dashboard/left_sidebar.rs` | Left sidebar |
| `src/components/dashboard/right_sidebar.rs` | Right sidebar |
| `src/components/posts/mod.rs` | Post components module |
| `src/components/posts/post_list.rs` | Infinite scroll post list |
| `src/components/posts/post_card.rs` | Post card component |
| `src/components/filters/mod.rs` | Filter components module |
| `src/components/filters/content_filter.rs` | Content type filter |
| `src/components/filters/feed_filter.rs` | Feed filter |
| `src/components/filters/sort_filter.rs` | Sort options |
| `src/components/search/mod.rs` | Search components module |
| `src/components/search/search_bar.rs` | Title/tag search |
| `src/components/search/user_search.rs` | User search dropdown |
| `src/components/widgets/mod.rs` | Widget components module |
| `src/components/widgets/profile_widget.rs` | Profile sidebar widget |
| `src/components/widgets/main_menu.rs` | Navigation menu |
| `style/dashboard.scss` | Dashboard styles |
| `style/posts.scss` | Post card styles |

### Modified Files

| File | Changes |
|------|---------|
| `src/app.rs` | Add `/dashboard` route |
| `src/models/mod.rs` | Export `post` module |
| `src/models/user.rs` | Add `UserInfo` struct (existing file) |
| `src/api/mod.rs` | Export `posts` module |
| `src/api/graphql.rs` | Add post queries/mutations |
| `src/components/mod.rs` | Export dashboard, posts, filters, widgets submodules |
| `src/pages/mod.rs` | Export dashboard page |
| `peer-web/Cargo.toml` | Add `web-sys` IntersectionObserver features |
| `src/pages/mod.rs` | Export dashboard page |

---

## UI/UX Requirements

### Visual Parity

- Three-column grid layout (sidebar / main / sidebar)
- Dark theme with CSS custom properties
- Fixed header with logo and title
- Collapsible left sidebar filters
- Sticky sidebars on scroll
- Post card grid with hover effects
- Loading spinner (animated logo)

### Post Card Behavior

1. **On Hover**
   - Subtle scale/shadow effect
   - Show action buttons

2. **On Click**
   - Open view post overlay/modal
   - Mark as viewed (API call)

3. **Content Types**
   - Image: Show cover image, hover shows play if video
   - Video: Show cover with play button overlay
   - Audio: Show music player graphic
   - Text: Typography-focused card

### Filter Behavior

- Accordion-style collapsible sections
- Filter changes trigger immediate reload
- Persist selections to localStorage
- Visual indicators of active filters

### Infinite Scroll

- Load 20 posts per batch
- IntersectionObserver on loader element
- Show loading spinner while fetching
- "No posts found" when empty

### Mobile Responsive

- Single-column layout on mobile
- Bottom navigation bar
- Swipeable filter drawer
- Touch-friendly interactions

### Accessibility

- Keyboard navigation for post cards
- Focus management for modal
- Screen reader announcements for loading/empty states
- Proper ARIA labels for filters

---

## Testing Plan

### Unit Tests

- [ ] Post model deserialization
- [ ] Filter state localStorage persistence
- [ ] Content type filter toggle logic
- [ ] Debounced search function

### Integration Tests (E2E)

- [ ] Page loads with authentication
- [ ] Unauthorized redirect to login
- [ ] Posts load on initial render
- [ ] Infinite scroll loads more posts
- [ ] Content filter shows/hides posts
- [ ] Feed filter (followers/following) works
- [ ] Sort order changes post order
- [ ] Title search filters posts
- [ ] User search shows dropdown
- [ ] User search click navigates to profile
- [ ] Like/dislike toggles correctly
- [ ] Post click opens view modal
- [ ] Filter state persists across reload
- [ ] Empty state displays correctly
- [ ] Loading state displays correctly
- [ ] Ad posts render with indicator

---

## Performance Considerations

### Optimizations

1. **Virtual Scrolling** (future enhancement)
   - For very long post lists
   - Only render visible posts

2. **Image Lazy Loading**
   - Use `loading="lazy"` on images
   - Placeholder/skeleton while loading

3. **Debounced Inputs**
   - Search inputs debounced 300ms
   - Filter changes debounced

4. **Memoization**
   - Post cards use keyed iteration
   - Expensive computations memoized

5. **Request Deduplication**
   - Prevent duplicate API calls during scroll
   - Cancel pending requests on filter change

### Bundle Size

- Lazy load view post modal
- Code-split dashboard from auth pages
- Minimize GraphQL query strings

---

## Dependencies

### Rust Crates

Already in use:
- `leptos` — framework
- `serde` / `serde_json` — serialization
- `reqwest` — HTTP client (SSR)
- `web-sys` — DOM APIs (already in Cargo.toml)

**`web-sys` features in `Cargo.toml`:** ✅ Already added — includes `IntersectionObserver`,
`IntersectionObserverInit`, `IntersectionObserverEntry`, and additional features for file
uploads and input handling.

### Shared Resources

From legacy:
- `css/dashboard.css` → `style/dashboard.scss` ✅
- `css/all-post.css` → merged into `style/dashboard.scss` (no separate `posts.scss`)
- `svg/` icons (Home, filter icons, content icons, etc.)
- `img/` placeholders

---

## Estimated Effort

| Phase | Effort |
|-------|--------|
| Phase 1: Models & Infrastructure | 3-4 hours |
| Phase 2: Posts API Layer | 3-4 hours |
| Phase 3: Dashboard Layout | 4-5 hours |
| Phase 4: Post Feed Components | 6-8 hours |
| Phase 5: Filter Components | 4-5 hours |
| Phase 6: Search Components | 3-4 hours |
| Phase 7: Sidebar Widgets | 4-5 hours |
| Styling & Polish | 4-6 hours |
| Testing | 4-6 hours |
| **Total** | **35-47 hours** |

---

## Open Questions

1. ~~**View Post Overlay vs Page**~~
   - ~~Should clicking a post open a modal overlay (like legacy) or navigate to `/post/:id`?~~
   - **Status:** Post click handler exists but is a TODO stub (logs to console). Needs implementation.

2. ~~**Real-time Updates**~~
   - ~~Should post interactions (likes) update in real-time for other users?~~
   - **Resolved:** Optimistic local state updates implemented. Real-time sync is future work.

3. ~~**Ad Frequency**~~
   - ~~How often should ads appear in feed?~~
   - **Resolved:** Ads inserted every 5 posts (`AD_INTERVAL = 5` in `post_list.rs`). Ads fetched once on mount (up to 50), cycled through.

4. ~~**Content Filtering**~~
   - ~~How to handle `MYGRANDMALIKES` vs `MYGRANDMAHATES` severity levels?~~
   - **Resolved:** Added `content_filter_by: Option<ContentFilterType>` to `list_posts`.
     Fetch user’s preference from `userPreferences.contentFilteringSeverityLevel` on load.

---

## Known Issues

1. **Missing imports in `user_search.rs` under `hydrate` feature**
   - `search_users` (from `crate::api::posts`) and `spawn_local` (from `leptos::task`) are used
     inside a `#[cfg(feature = "hydrate")]` block but not imported. Default `cargo check` passes
     because it compiles without `hydrate`, but the client build will fail.

2. **ProfileWidget returns placeholder data**
   - `profile_widget.rs` has a `Resource` that always returns `None::<UserInfo>` with a TODO
     comment. It needs to read the user ID from the auth context and call `get_user_info`.

3. **Post click handler is a stub**
   - `post_card.rs` `on_click` just logs the post ID. The view-post overlay or navigation
     has not been wired up yet.

4. **GraphQL query omits some `Post` model fields**
   - `LIST_POSTS_QUERY` does not request `isreported`, `amounttrending`, `hasActiveReports`,
     `visibilityStatus`, or `isHiddenForUsers`. The `Post` struct has `#[serde(default)]` on
     these fields so deserialization won't break, but the values will always be defaults.

---

## Changelog

### 2026-04-14
- Updated status to 🟡 Mostly Implemented
- Marked completed scope items (16/18 done)
- Documented 4 known issues found during code review
- Resolved open questions #1–#3
- Noted `posts.scss` was merged into `dashboard.scss`
- Confirmed `web-sys` Cargo features are in place

### 2026-04-10 (v2)
- Fixed API response models to use `meta: DefaultResponse` envelope matching actual backend
- Added missing `listPosts` query parameters: `IgnorList`, `from`, `to`
- Added missing response codes to documentation
- Added `isfriend` field to `PostUser`
- Added `ContentFilterType` and `IgnoreOption` enums to models
- Added `content_filter_by` parameter to `list_posts` server function
- Fixed Leptos 0.8 API usage:
  - Replaced `create_memo` with `Memo::new`
  - Replaced `create_debounced_action` with custom `web-sys` debounce
  - Fixed `IntersectionObserver` setup with proper `web-sys` API and cleanup
  - Added `on_cleanup` to disconnect observer on unmount
- Fixed user search:
  - Added debouncing (was missing)
  - Replaced fragile `on:blur` delay with `on:mousedown` prevention
- Added `AdvertisementPost`, `AdvertisementInfo`, `AdListResponse` types
- Added `FeedItem` enum for unified feed rendering (posts + ads)
- Updated files table: `src/models/user.rs` is modified (not new)
- Added structural notes about component organization change
- Added required `web-sys` features to dependencies section
- Removed unnecessary `gloo-*` crate suggestions (using `web-sys` directly)

### 2026-04-10 (v1)
- Initial planning document created
- Analyzed legacy dashboard implementation
- Defined component structure and API requirements
