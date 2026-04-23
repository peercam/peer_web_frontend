# View Post Implementation Plan

**Feature:** View Post  
**Priority:** #4 (after Dashboard)  
**Status:** � ~95% Implemented — polish & testing remain  
**Created:** 2026-04-12  
**Updated:** 2026-04-14

---

## Overview

Implement the single post view page for the Leptos frontend. This is a critical page for SEO and sharing, as it supports **guest mode** (unauthenticated viewing) while allowing authenticated users full interaction capabilities.

### Goals

1. Full parity with legacy `post.php` user experience
2. Guest-accessible view (no auth required for reading)
3. Deep-linking support (`/post/{id}` or `/p/{id}`)
4. Full post media display (image gallery, video, audio)
5. Comments with nested replies
6. Post interactions (like, dislike, save, share, report)
7. Mobile app deep-link redirect for mobile users
8. SEO-optimized with dynamic meta tags
9. Modal overlay mode when navigating from dashboard

---

## Scope

### In Scope

- [ ] View post page (`/post/:id` route)
- [ ] Guest mode (unauthenticated reading via `guestListPost`)
- [ ] Authenticated mode (full interactions via `listPosts`)
- [ ] Post media components:
  - [ ] Image gallery with slider/modal
  - [ ] Video player
  - [ ] Audio player
  - [ ] Text content display
- [ ] Post metadata (title, description, tags, timestamp)
- [ ] Post author header with follow button
- [ ] Post statistics (views, likes, dislikes, comments)
- [ ] Post actions (like, dislike, save, report)
- [ ] Share functionality (copy link, WhatsApp, Telegram)
- [ ] Comments section:
  - [ ] Top-level comments list with infinite scroll
  - [ ] Nested replies (single level)
  - [ ] Comment creation form
  - [ ] Like comment functionality
- [ ] Modal overlay mode (when clicking post from feed)
- [ ] Mobile deep-link redirect
- [ ] Dynamic SEO meta tags (SSR)
- [ ] Loading states and skeletons
- [ ] Error states (post not found, deleted)
- [ ] Close button / back navigation

### Out of Scope (Future Work)

- Comment reporting
- Boost post functionality
- Download media
- Edit post (author only)
- Delete post (author only)
- Advanced moderation tools

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `post.php` | Guest view page template |
| `template-parts/content-parts/view-post.php` | View post modal/overlay template |
| `js/guestpost.js` | Guest post fetching, mobile deep-link logic |
| `js/posts.js` | Post rendering, like/dislike/save handlers |
| `js/comments.js` | Comment fetching, creation, replies, likes |
| `css/view-post.css` | View post modal styles |
| `css/post.css` | Post page-specific styles |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│ Modal Overlay (backdrop blur)                              │
├───────────────────────────────────┬────────────────────────┤
│                                   │                        │
│   LEFT PANEL (60%)                │   RIGHT PANEL (40%)    │
│                                   │                        │
│   ┌─────────────────────────┐     │   ┌──────────────────┐ │
│   │                         │     │   │ User Avatar/Name │ │
│   │   POST MEDIA            │     │   │ Follow Button    │ │
│   │   (Gallery/Video/Audio) │     │   └──────────────────┘ │
│   │                         │     │                        │
│   └─────────────────────────┘     │   ┌──────────────────┐ │
│                                   │   │ Title            │ │
│   ┌─────────────────────────┐     │   │ Timestamp        │ │
│   │ Views │ Likes │ Comments│     │   │ Description      │ │
│   │ Report │ Share │ More   │     │   │ #tags            │ │
│   └─────────────────────────┘     │   └──────────────────┘ │
│                                   │                        │
│                                   │   ┌──────────────────┐ │
│                                   │   │ COMMENTS         │ │
│                                   │   │ (scrollable)     │ │
│                                   │   │                  │ │
│                                   │   └──────────────────┘ │
│                                   │                        │
│                                   │   ┌──────────────────┐ │
│                                   │   │ Comment Input    │ │
│                                   │   └──────────────────┘ │
├───────────────────────────────────┴────────────────────────┤
│ [X] Close Button (top-right)                               │
└────────────────────────────────────────────────────────────┘
```

### Mobile Layout

```
┌──────────────────────────┐
│ [←] Post      [⋮] More   │
├──────────────────────────┤
│                          │
│   POST MEDIA (full-width)│
│                          │
├──────────────────────────┤
│ 👤 Username  #slug       │
│ [Follow]                 │
├──────────────────────────┤
│ Title                    │
│ Description              │
│ #tags                    │
├──────────────────────────┤
│ 👁 123  ❤️ 45  💬 12     │
│ [Like] [Dislike] [Share] │
├──────────────────────────┤
│ Comments (12)            │
│ ─────────────────────    │
│ Comment 1                │
│ Comment 2                │
│ ...                      │
├──────────────────────────┤
│ [Write a comment...]     │
└──────────────────────────┘
```

### Key Features

1. **Guest Mode**
   - Uses `guestListPost` query (no auth required)
   - Redirects to app store on mobile
   - Shows "Sign Up" CTA
   - No interactions allowed

2. **Authenticated Mode**
   - Full interactions (like, dislike, save, report)
   - Comment creation
   - Follow author

3. **Image Gallery**
   - Multiple images with slider
   - Fullscreen modal view
   - Thumbnail navigation
   - Swipe on mobile

4. **Video Player**
   - Native HTML5 video
   - Cover image thumbnail
   - Play/pause controls

5. **Audio Player**
   - Custom audio controls
   - Waveform visualization (optional)

6. **Comments**
   - Paginated loading (10 per batch)
   - Nested replies (1 level)
   - Like comment
   - Reply to comment
   - Infinite scroll

7. **Share**
   - Copy link to clipboard
   - WhatsApp deep link
   - Telegram deep link
   - Native share API (mobile)

8. **Mobile Deep Link**
   - Detect Android/iOS
   - Try `peer://post/{id}` deep link
   - Fallback to app store

---

## Backend API Reference

### `guestListPost` Query (No Auth)

```graphql
query GuestListPost($postid: ID!) {
  guestListPost(postid: $postid) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
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
      tags
      user {
        id
        username
        slug
        img
      }
    }
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11501` | Post retrieved successfully |
| `30209` | Invalid post UUID |
| `31510` | Post not found |

### `listPosts` Query (With Auth, Single Post)

```graphql
query ListPost($postid: ID!) {
  listPosts(postid: $postid, limit: 1) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
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

### `listComments` Query

```graphql
query ListComments($postid: ID!, $commentOffset: Int, $commentLimit: Int) {
  listComments(
    postid: $postid
    commentOffset: $commentOffset
    commentLimit: $commentLimit
  ) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      commentid
      userid
      postid
      parentid
      content
      createdat
      amountlikes
      amountreplies
      isliked
      user {
        id
        username
        slug
        img
        isfollowed
        isfollowing
      }
    }
  }
}
```

### `listChildComments` Query

```graphql
query ListChildComments($parent: ID!, $offset: Int, $limit: Int) {
  listChildComments(parent: $parent, offset: $offset, limit: $limit) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      commentid
      userid
      postid
      parentid
      content
      createdat
      amountlikes
      amountreplies
      isliked
      user {
        id
        username
        slug
        img
        isfollowed
        isfollowing
      }
    }
  }
}
```

### `createComment` Mutation

```graphql
mutation CreateComment($postid: ID!, $content: String!, $parentid: ID) {
  createComment(
    action: COMMENT
    postid: $postid
    content: $content
    parentid: $parentid
  ) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      commentid
      userid
      content
      createdat
      amountlikes
      amountreplies
      isliked
      postid
      parentid
      user {
        id
        username
        slug
        img
        isfollowed
        isfollowing
      }
    }
  }
}
```

### `likeComment` Mutation

```graphql
mutation LikeComment($commentid: ID!) {
  likeComment(commentid: $commentid) {
    status
    ResponseCode
    ResponseMessage
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

---

## Implementation Plan

### Phase 1: Core Models & API Layer

#### 1.1 Comment Models (`src/models/comment.rs`)

```rust
use serde::Deserialize;

use crate::models::DefaultResponse;

/// User embedded in comments.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentUser {
    pub id: String,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    pub isfollowed: bool,
    pub isfollowing: bool,
}

/// Comment data from API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub commentid: String,
    pub userid: String,
    pub postid: String,
    pub parentid: Option<String>,
    pub content: String,
    pub createdat: String,
    pub amountlikes: i32,
    pub amountreplies: i32,
    pub isliked: bool,
    pub user: CommentUser,
}

/// Comment list response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Vec<Comment>,
}

impl CommentListResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}
```

#### 1.2 Comments API (`src/api/comments.rs`)

```rust
use leptos::prelude::*;
use serde::Serialize;

use crate::models::comment::{Comment, CommentListResponse};

/// Fetch top-level comments for a post.
#[server(ListComments, "/api")]
pub async fn list_comments(
    post_id: String,
    offset: i32,
    limit: i32,
) -> Result<CommentListResponse, ServerFnError>;

/// Fetch replies to a comment.
#[server(ListChildComments, "/api")]
pub async fn list_child_comments(
    parent_id: String,
    offset: i32,
    limit: i32,
) -> Result<CommentListResponse, ServerFnError>;

/// Create a new comment or reply.
#[server(CreateComment, "/api")]
pub async fn create_comment(
    post_id: String,
    content: String,
    parent_id: Option<String>,
) -> Result<Comment, ServerFnError>;

/// Like a comment.
///
/// Production peergamma's `likeComment` is a **toggle**: a second call by the
/// same user removes the like. There is no `unlikeComment` resolver.
#[server(LikeComment, "/api")]
pub async fn like_comment(comment_id: String) -> Result<(), ServerFnError>;

/// Unlike a comment (client-side wrapper that re-issues `likeComment` to
/// toggle the like off — kept for call-site clarity in UI handlers).
#[server(UnlikeComment, "/api")]
pub async fn unlike_comment(comment_id: String) -> Result<(), ServerFnError>;
```

#### 1.3 Guest Post API (`src/api/posts.rs` addition)

```rust
/// Fetch a single post for guest viewing (no auth required).
#[server(GuestGetPost, "/api")]
pub async fn guest_get_post(post_id: String) -> Result<Post, ServerFnError> {
    use crate::api::graphql::{query, GuestPostData, GUEST_POST_QUERY};

    let vars = GetPostVars { postid: post_id };
    let data: GuestPostData = query(GUEST_POST_QUERY, vars, None).await?;
    
    data.guest_list_post
        .affected_rows
        .ok_or_else(|| ServerFnError::new("Post not found"))
}
```

#### 1.4 GraphQL Queries (`src/api/graphql.rs` additions)

```rust
pub const GUEST_POST_QUERY: &str = r#"
    query GuestListPost($postid: ID!) {
        guestListPost(postid: $postid) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
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
                tags
                user {
                    id
                    username
                    slug
                    img
                }
            }
        }
    }
"#;

pub const LIST_COMMENTS_QUERY: &str = r#"
    query ListComments($postid: ID!, $commentOffset: Int, $commentLimit: Int) {
        listComments(
            postid: $postid
            commentOffset: $commentOffset
            commentLimit: $commentLimit
        ) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            counter
            affectedRows {
                commentid
                userid
                postid
                parentid
                content
                createdat
                amountlikes
                amountreplies
                isliked
                user {
                    id
                    username
                    slug
                    img
                    isfollowed
                    isfollowing
                }
            }
        }
    }
"#;

pub const LIST_CHILD_COMMENTS_QUERY: &str = r#"
    query ListChildComments($parent: ID!, $offset: Int, $limit: Int) {
        listChildComments(parent: $parent, offset: $offset, limit: $limit) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            counter
            affectedRows {
                commentid
                userid
                postid
                parentid
                content
                createdat
                amountlikes
                amountreplies
                isliked
                user {
                    id
                    username
                    slug
                    img
                    isfollowed
                    isfollowing
                }
            }
        }
    }
"#;

pub const CREATE_COMMENT_MUTATION: &str = r#"
    mutation CreateComment($postid: ID!, $content: String!, $parentid: ID) {
        createComment(
            action: COMMENT
            postid: $postid
            content: $content
            parentid: $parentid
        ) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            counter
            affectedRows {
                commentid
                userid
                content
                createdat
                amountlikes
                amountreplies
                isliked
                postid
                parentid
                user {
                    id
                    username
                    slug
                    img
                    isfollowed
                    isfollowing
                }
            }
        }
    }
"#;

pub const LIKE_COMMENT_MUTATION: &str = r#"
    mutation LikeComment($commentid: ID!) {
        likeComment(commentid: $commentid) {
            status
            ResponseCode
            ResponseMessage
        }
    }
"#;
```

### Phase 2: View Post Page

#### 2.1 Route Setup (`src/app.rs`)

```rust
// Add route in app router
<Route path="/post/:id" view=ViewPostPage/>
<Route path="/p/:id" view=ViewPostPage/>  // Short URL alias
```

#### 2.2 View Post Page (`src/pages/view_post.rs`)

```rust
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::api::posts::{guest_get_post, get_post};
use crate::components::view_post::{
    PostMedia, PostHeader, PostContent, PostActions, Comments
};
use crate::state::auth::use_auth_context;

/// View single post page.
///
/// Supports both guest mode (no auth) and authenticated mode.
#[component]
pub fn ViewPostPage() -> impl IntoView {
    let params = use_params_map();
    let auth = use_auth_context();
    
    let post_id = move || {
        params.get().get("id").unwrap_or_default()
    };
    
    // Fetch post based on auth state
    let post_resource = Resource::new(
        move || (post_id(), auth.is_authenticated()),
        |(id, is_auth)| async move {
            if is_auth {
                get_post(id).await
            } else {
                guest_get_post(id).await
            }
        }
    );

    view! {
        <Suspense fallback=move || view! { <PostSkeleton/> }>
            {move || post_resource.get().map(|result| {
                match result {
                    Ok(post) => view! {
                        <Title text=format!("{} - Peer Network", post.title)/>
                        <ViewPostContent post=post is_guest=!auth.is_authenticated()/>
                    }.into_any(),
                    Err(_) => view! {
                        <Title text="Post Not Found - Peer Network"/>
                        <PostNotFound/>
                    }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn ViewPostContent(post: Post, is_guest: bool) -> impl IntoView {
    view! {
        <div id="viewpost" class="viewpost">
            <div id="viewpost-container" class="inner-container">
                <div class="viewpost-left">
                    <PostMedia post=post.clone()/>
                    <PostActions post=post.clone() is_guest=is_guest/>
                </div>
                <div class="viewpost-right">
                    <PostHeader user=post.user.clone() is_guest=is_guest/>
                    <PostContent post=post.clone()/>
                    <Comments post_id=post.id.clone() is_guest=is_guest/>
                </div>
                <CloseButton/>
            </div>
        </div>
        
        // Guest mode CTA
        <Show when=move || is_guest>
            <GuestModeBanner/>
        </Show>
    }
}

#[component]
fn PostNotFound() -> impl IntoView {
    view! {
        <div class="post-not-found">
            <h2 class="xxl_font_size">"Post Not Found"</h2>
            <p class="md_font_size">"The post you are looking for does not exist or has been removed."</p>
            <a href="/dashboard" class="button btn-blue">"Go to Dashboard"</a>
        </div>
    }
}

#[component]
fn GuestModeBanner() -> impl IntoView {
    view! {
        <div class="view-only-mode">
            <div class="view-only-mode-heading">
                <svg>/* eye icon */</svg>
                <span class="xl_font_size bold">"View-only mode"</span>
            </div>
            <p class="md_font_size txt-color-gray">"Sign up to interact and access everything!"</p>
            <a href="/register" class="button btn-blue">"Sign Up"</a>
        </div>
    }
}
```

### Phase 3: Post Media Components

#### 3.1 Post Media (`src/components/view_post/post_media.rs`)

```rust
use leptos::prelude::*;

use crate::models::post::{ContentType, Post};

/// Post media display (image gallery, video, audio, or text).
#[component]
pub fn PostMedia(post: Post) -> impl IntoView {
    match post.contenttype {
        ContentType::Image => view! { <ImageGallery post=post/> }.into_any(),
        ContentType::Video => view! { <VideoPlayer post=post/> }.into_any(),
        ContentType::Audio => view! { <AudioPlayer post=post/> }.into_any(),
        ContentType::Text => view! { <TextPost post=post/> }.into_any(),
    }
}

/// Image gallery with slider.
#[component]
fn ImageGallery(post: Post) -> impl IntoView {
    let images = parse_media_urls(&post.media);
    let current_index = RwSignal::new(0usize);
    let show_modal = RwSignal::new(false);

    view! {
        <div class="post_gallery">
            // Main image display
            <div class="gallery-main">
                <img
                    src=move || images.get(current_index.get()).cloned().unwrap_or_default()
                    alt=post.title.clone()
                    on:click=move |_| show_modal.set(true)
                />
                
                // Navigation arrows (if multiple images)
                <Show when=move || images.len() > 1>
                    <button
                        class="gallery-nav prev"
                        on:click=move |_| {
                            current_index.update(|i| {
                                *i = if *i == 0 { images.len() - 1 } else { *i - 1 };
                            });
                        }
                    >
                        <i class="peer-icon peer-icon-arrow-left"/>
                    </button>
                    <button
                        class="gallery-nav next"
                        on:click=move |_| {
                            current_index.update(|i| {
                                *i = (*i + 1) % images.len();
                            });
                        }
                    >
                        <i class="peer-icon peer-icon-arrow-right"/>
                    </button>
                </Show>
                
                // Dots indicator
                <Show when=move || images.len() > 1>
                    <div class="gallery-dots">
                        {images.iter().enumerate().map(|(i, _)| {
                            view! {
                                <span
                                    class="gallery-dot"
                                    class:active=move || current_index.get() == i
                                    on:click=move |_| current_index.set(i)
                                />
                            }
                        }).collect_view()}
                    </div>
                </Show>
            </div>
            
            // Thumbnails
            <Show when=move || images.len() > 1>
                <div class="gallery-thumbnails">
                    {images.iter().enumerate().map(|(i, url)| {
                        let url = url.clone();
                        view! {
                            <img
                                src=url
                                class:active=move || current_index.get() == i
                                on:click=move |_| current_index.set(i)
                            />
                        }
                    }).collect_view()}
                </div>
            </Show>
        </div>
        
        // Fullscreen modal
        <Show when=move || show_modal.get()>
            <ImageModal
                images=images.clone()
                current_index=current_index
                on_close=move || show_modal.set(false)
            />
        </Show>
    }
}

/// Video player component.
#[component]
fn VideoPlayer(post: Post) -> impl IntoView {
    let video_url = post.media.unwrap_or_default();
    let cover_url = post.cover.unwrap_or_default();

    view! {
        <div class="post_gallery video-container">
            <video
                controls
                poster=cover_url
                preload="metadata"
            >
                <source src=video_url type="video/mp4"/>
                "Your browser does not support the video tag."
            </video>
        </div>
    }
}

/// Audio player component.
#[component]
fn AudioPlayer(post: Post) -> impl IntoView {
    let audio_url = post.media.unwrap_or_default();
    let cover_url = post.cover.unwrap_or_else(|| "/svg/audio-placeholder.svg".to_string());

    view! {
        <div class="post_gallery audio-container">
            <img src=cover_url alt="Audio cover" class="audio-cover"/>
            <audio controls>
                <source src=audio_url type="audio/mpeg"/>
                "Your browser does not support the audio element."
            </audio>
        </div>
    }
}

/// Text-only post display.
#[component]
fn TextPost(post: Post) -> impl IntoView {
    view! {
        <div class="post_gallery text-container">
            <div class="text-content">
                <p>{post.mediadescription.unwrap_or_default()}</p>
            </div>
        </div>
    }
}
```

### Phase 4: Comments Components

#### 4.1 Comments Container (`src/components/view_post/comments.rs`)

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::comments::{create_comment, list_comments};
use crate::models::comment::Comment;

const COMMENTS_PER_PAGE: i32 = 10;

/// Comments section with infinite scroll.
#[component]
pub fn Comments(post_id: String, is_guest: bool) -> impl IntoView {
    let comments = RwSignal::new(Vec::<Comment>::new());
    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let total_count = RwSignal::new(0i32);
    
    let post_id_clone = post_id.clone();

    // Load comments
    let load_comments = move || {
        if is_loading.get() || !has_more.get() {
            return;
        }

        is_loading.set(true);
        let id = post_id_clone.clone();
        let current_offset = offset.get();

        spawn_local(async move {
            match list_comments(id, current_offset, COMMENTS_PER_PAGE).await {
                Ok(response) => {
                    let new_comments = response.affected_rows;
                    let has_new = !new_comments.is_empty();
                    
                    total_count.set(response.counter);

                    if has_new {
                        comments.update(|c| c.extend(new_comments));
                        offset.update(|o| *o += COMMENTS_PER_PAGE);
                    }

                    has_more.set(has_new && response.counter > current_offset + COMMENTS_PER_PAGE);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to load comments: {:?}", e);
                }
            }
            is_loading.set(false);
        });
    };

    // Initial load
    Effect::new(move |_| {
        if comments.get().is_empty() {
            load_comments();
        }
    });

    view! {
        <div class="comments-container">
            <div class="comments-header">
                <h3 class="cmt_head xxl_font_size bold">"Comments"</h3>
                <span class="comment_total md_font_size">
                    <i class="peer-icon peer-icon-comment-dot"/>
                    <span class="comment_count">{move || total_count.get()}</span>
                </span>
            </div>
            
            <div class="comment_list">
                <div id="comments" class="inner">
                    <For
                        each=move || comments.get()
                        key=|c| c.commentid.clone()
                        children=move |comment| {
                            view! {
                                <CommentItem
                                    comment=comment
                                    is_guest=is_guest
                                />
                            }
                        }
                    />
                    
                    <Show when=move || is_loading.get()>
                        <div class="comment-loading">"Loading..."</div>
                    </Show>
                    
                    <Show when=move || !is_loading.get() && has_more.get()>
                        <button
                            class="load-more-comments"
                            on:click=move |_| load_comments()
                        >
                            "Load more comments"
                        </button>
                    </Show>
                </div>
            </div>
            
            // Comment input (authenticated only)
            <Show when=move || !is_guest>
                <CommentInput
                    post_id=post_id.clone()
                    on_comment_added=move |new_comment| {
                        comments.update(|c| c.insert(0, new_comment));
                        total_count.update(|t| *t += 1);
                    }
                />
            </Show>
        </div>
    }
}

/// Single comment item.
#[component]
fn CommentItem(comment: Comment, is_guest: bool) -> impl IntoView {
    let is_liked = RwSignal::new(comment.isliked);
    let like_count = RwSignal::new(comment.amountlikes);
    let show_replies = RwSignal::new(false);
    let show_reply_form = RwSignal::new(false);
    
    let comment_id = comment.commentid.clone();
    let has_replies = comment.amountreplies > 0;
    let img_src = comment.user.img.unwrap_or_else(|| "/svg/noname.svg".to_string());
    let profile_url = format!("/profile/{}", comment.user.slug);

    let on_like = move |_| {
        if is_guest {
            return;
        }
        
        let id = comment_id.clone();
        let currently_liked = is_liked.get();

        // Optimistic update
        if currently_liked {
            is_liked.set(false);
            like_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_liked.set(true);
            like_count.update(|c| *c += 1);
        }

        spawn_local(async move {
            if currently_liked {
                let _ = unlike_comment(id).await;
            } else {
                let _ = like_comment(id).await;
            }
        });
    };

    view! {
        <div class="comment_item" id=comment.commentid.clone()>
            <div class="commenter-pic">
                <img class="profile-picture" src=img_src/>
            </div>
            <div class="comment_body">
                <div class="commenter_info">
                    <a href=profile_url>
                        <span class="cmt_userName md_font_size bold">{comment.user.username}</span>
                    </a>
                    <span class="timeagao txt-color-gray">
                        {format_time_ago(&comment.createdat)}
                    </span>
                </div>
                <div class="comment_text">{comment.content}</div>
                
                <div class="comment_reply_container">
                    <Show when=move || !is_guest>
                        <span
                            class="reply_btn"
                            on:click=move |_| show_reply_form.update(|s| *s = !*s)
                        >
                            <a class="md_font_size bold">"Reply"</a>
                        </span>
                    </Show>
                    
                    <Show when=move || has_replies>
                        <span
                            class="show_reply txt-color-gray"
                            on:click=move |_| show_replies.update(|s| *s = !*s)
                        >
                            {move || if show_replies.get() {
                                "Hide replies".to_string()
                            } else {
                                format!("Show {} replies...", comment.amountreplies)
                            }}
                        </span>
                    </Show>
                </div>
                
                // Reply form
                <Show when=move || show_reply_form.get()>
                    <ReplyForm
                        post_id=comment.postid.clone()
                        parent_id=comment.commentid.clone()
                        on_reply_added=move |_| {
                            show_reply_form.set(false);
                            // Optionally refresh replies
                        }
                    />
                </Show>
                
                // Nested replies
                <Show when=move || show_replies.get()>
                    <ChildComments
                        parent_id=comment.commentid.clone()
                        is_guest=is_guest
                    />
                </Show>
            </div>
            
            // Like button
            <div
                class="comment_like"
                class:liked=move || is_liked.get()
                on:click=on_like
            >
                <span>{move || like_count.get()}</span>
            </div>
        </div>
    }
}

/// Comment input form.
#[component]
fn CommentInput<F>(post_id: String, on_comment_added: F) -> impl IntoView
where
    F: Fn(Comment) + 'static,
{
    let content = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let post_id_clone = post_id.clone();

    let on_submit = move |_| {
        let text = content.get().trim().to_string();
        if text.is_empty() || is_submitting.get() {
            return;
        }

        is_submitting.set(true);
        let id = post_id_clone.clone();

        spawn_local(async move {
            match create_comment(id, text, None).await {
                Ok(new_comment) => {
                    content.set(String::new());
                    on_comment_added(new_comment);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to create comment: {:?}", e);
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div id="post_comment" class="post_comment">
            <textarea
                placeholder="Share your thoughts..."
                prop:value=move || content.get()
                on:input=move |ev| content.set(event_target_value(&ev))
                disabled=move || is_submitting.get()
            />
            <button
                on:click=on_submit
                disabled=move || is_submitting.get() || content.get().trim().is_empty()
            >
                <i class="peer-icon peer-icon-arrow-right"/>
            </button>
        </div>
    }
}
```

### Phase 5: Post Actions & Share

#### 5.1 Post Actions (`src/components/view_post/post_actions.rs`)

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::posts::post_action;
use crate::models::post::{Post, PostActionType};

/// Post action bar (like, dislike, save, share, report).
#[component]
pub fn PostActions(post: Post, is_guest: bool) -> impl IntoView {
    let is_liked = RwSignal::new(post.isliked);
    let is_disliked = RwSignal::new(post.isdisliked);
    let is_saved = RwSignal::new(post.issaved);
    let like_count = RwSignal::new(post.amountlikes);
    let dislike_count = RwSignal::new(post.amountdislikes);
    let show_share = RwSignal::new(false);
    let show_more = RwSignal::new(false);

    let post_id = post.id.clone();
    let post_id_like = post.id.clone();
    let post_id_dislike = post.id.clone();
    let post_id_save = post.id.clone();

    let on_like = move |_| {
        if is_guest { return; }
        
        let id = post_id_like.clone();
        let currently_liked = is_liked.get();
        let currently_disliked = is_disliked.get();

        // Optimistic update
        if currently_liked {
            is_liked.set(false);
            like_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_liked.set(true);
            like_count.update(|c| *c += 1);
            if currently_disliked {
                is_disliked.set(false);
                dislike_count.update(|c| *c = (*c - 1).max(0));
            }
        }

        spawn_local(async move {
            let action = if currently_liked {
                PostActionType::Unlike
            } else {
                PostActionType::Like
            };
            let _ = post_action(id, action).await;
        });
    };

    let on_dislike = move |_| {
        if is_guest { return; }
        
        let id = post_id_dislike.clone();
        let currently_liked = is_liked.get();
        let currently_disliked = is_disliked.get();

        if currently_disliked {
            is_disliked.set(false);
            dislike_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_disliked.set(true);
            dislike_count.update(|c| *c += 1);
            if currently_liked {
                is_liked.set(false);
                like_count.update(|c| *c = (*c - 1).max(0));
            }
        }

        spawn_local(async move {
            let action = if currently_disliked {
                PostActionType::Undislike
            } else {
                PostActionType::Dislike
            };
            let _ = post_action(id, action).await;
        });
    };

    let on_save = move |_| {
        if is_guest { return; }
        
        let id = post_id_save.clone();
        let currently_saved = is_saved.get();
        is_saved.set(!currently_saved);

        spawn_local(async move {
            let action = if currently_saved {
                PostActionType::Unsave
            } else {
                PostActionType::Save
            };
            let _ = post_action(id, action).await;
        });
    };

    view! {
        <div class="postview_footer">
            <div class="social md_font_size">
                <div class="post-view txt-color-gray">
                    <i class="peer-icon peer-icon-eye-open"/>
                    <span>{post.amountviews}</span>
                </div>
                
                <div
                    class="post-like"
                    class:active=move || is_liked.get()
                    class:disabled=is_guest
                    on:click=on_like
                >
                    <i class="peer-icon peer-icon-like"/>
                    <span>{move || like_count.get()}</span>
                </div>
                
                <div
                    class="post-dislike"
                    class:active=move || is_disliked.get()
                    class:disabled=is_guest
                    on:click=on_dislike
                >
                    <i class="peer-icon peer-icon-dislike"/>
                    <span>{move || dislike_count.get()}</span>
                </div>
                
                <div class="post-comments">
                    <i class="peer-icon peer-icon-comment-alt"/>
                    <span>{post.amountcomments}</span>
                </div>
            </div>
            
            <div class="more md_font_size">
                <ul>
                    <li>
                        <a href="#" class="morebtn" on:click=move |e| {
                            e.prevent_default();
                            show_more.update(|s| *s = !*s);
                        }>
                            <span class="textval">"More"</span>
                            <span class="dots"/>
                        </a>
                    </li>
                    <Show when=move || show_more.get()>
                        <ul class="sublist">
                            <Show when=move || !is_guest>
                                <li>
                                    <a href="#" class="reportpost">
                                        <i class="peer-icon peer-icon-flag-fill"/>
                                        <span>"Report post"</span>
                                    </a>
                                </li>
                            </Show>
                            <li class="sharelinks">
                                <a href="#" class="share" on:click=move |e| {
                                    e.prevent_default();
                                    show_share.set(true);
                                }>
                                    <i class="peer-icon peer-icon-share-link"/>
                                    " Share"
                                </a>
                            </li>
                            <Show when=move || !is_guest>
                                <li>
                                    <a href="#"
                                        class="save"
                                        class:active=move || is_saved.get()
                                        on:click=move |e| {
                                            e.prevent_default();
                                            on_save(());
                                        }
                                    >
                                        <i class="peer-icon peer-icon-save-fill"/>
                                        {move || if is_saved.get() { " Saved" } else { " Save" }}
                                    </a>
                                </li>
                            </Show>
                        </ul>
                    </Show>
                </ul>
            </div>
        </div>
        
        // Share modal
        <Show when=move || show_share.get()>
            <ShareModal
                post_id=post_id.clone()
                on_close=move || show_share.set(false)
            />
        </Show>
    }
}
```

#### 5.2 Share Modal (`src/components/view_post/share_modal.rs`)

```rust
use leptos::prelude::*;

/// Share modal with copy link, WhatsApp, Telegram.
#[component]
pub fn ShareModal(post_id: String, on_close: impl Fn() + 'static) -> impl IntoView {
    let post_url = format!("{}/post/{}", get_base_url(), post_id);
    let copied = RwSignal::new(false);

    let copy_to_clipboard = move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen_futures::JsFuture;
            let url = post_url.clone();
            spawn_local(async move {
                if let Some(clipboard) = web_sys::window()
                    .and_then(|w| w.navigator().clipboard())
                {
                    let _ = JsFuture::from(clipboard.write_text(&url)).await;
                    copied.set(true);
                    // Reset after 2 seconds
                    set_timeout(move || copied.set(false), Duration::from_secs(2));
                }
            });
        }
    };

    let whatsapp_url = format!(
        "https://api.whatsapp.com/send?text={}",
        url_encode(&post_url)
    );
    let telegram_url = format!(
        "https://t.me/share/url?url={}",
        url_encode(&post_url)
    );

    view! {
        <div id="share-link-box" class="share-post-box">
            <div id="closeSharebox" class="btClose" on:click=move |_| on_close()>"X"</div>
            
            <div class="share-post-subbox">
                <h3 class="xl_font_size">"Share"</h3>
                <div class="share-post-apps">
                    <a href=whatsapp_url target="_blank" rel="noopener noreferrer">
                        <span class="share-app">
                            <span class="app-icon">
                                /* WhatsApp SVG icon */
                            </span>
                            <span class="app-name">"WhatsApp"</span>
                        </span>
                    </a>
                    <a href=telegram_url target="_blank" rel="noopener noreferrer">
                        <span class="share-app telegram">
                            <span class="app-icon">
                                /* Telegram SVG icon */
                            </span>
                            <span class="app-name">"Telegram"</span>
                        </span>
                    </a>
                </div>
            </div>
            
            <div class="share-post-subbox">
                <h3 class="xl_font_size">"Link"</h3>
                <div class="share-post-link">
                    <input
                        type="text"
                        class="share-link-input"
                        value=post_url.clone()
                        readonly
                    />
                    <button
                        class="copy-link-btn btn-white"
                        on:click=copy_to_clipboard
                    >
                        <span>{move || if copied.get() { "Copied!" } else { "Copy" }}</span>
                        <i class="peer-icon peer-icon-copy-alt"/>
                    </button>
                </div>
            </div>
        </div>
    }
}
```

### Phase 6: Mobile Deep Link & SEO

#### 6.1 Mobile Deep Link Hook (`src/hooks/use_mobile_redirect.rs`)

```rust
use leptos::prelude::*;

/// Detect mobile and attempt deep link redirect.
pub fn use_mobile_redirect(post_id: &str) {
    #[cfg(feature = "hydrate")]
    {
        let id = post_id.to_string();
        
        Effect::new(move |_| {
            use wasm_bindgen::JsCast;
            use web_sys::window;
            
            let Some(window) = window() else { return };
            let navigator = window.navigator();
            let user_agent = navigator.user_agent().unwrap_or_default().to_lowercase();
            
            let is_android = user_agent.contains("android");
            let is_ios = user_agent.contains("iphone") 
                || user_agent.contains("ipad") 
                || user_agent.contains("ipod");
            
            if is_android || is_ios {
                let deep_link = format!("peer://post/{}", id);
                let _ = window.location().set_href(&deep_link);
                
                // Fallback after delay (if app doesn't open)
                // Note: This is handled by the app store redirect in the UI
            }
        });
    }
}
```

#### 6.2 SEO Meta Tags (SSR)

```rust
use leptos_meta::{Title, Meta};

/// Dynamic SEO meta tags for post.
#[component]
fn PostSeoMeta(post: &Post) -> impl IntoView {
    let title = format!("{} - Peer Network", post.title);
    let description = post.mediadescription.clone()
        .unwrap_or_else(|| post.title.clone());
    let image = post.cover.clone()
        .or_else(|| post.media.clone())
        .unwrap_or_else(|| "/svg/og-default.svg".to_string());
    let url = format!("{}/post/{}", get_base_url(), post.id);

    view! {
        <Title text=title.clone()/>
        <Meta name="description" content=description.clone()/>
        
        // Open Graph
        <Meta property="og:title" content=title.clone()/>
        <Meta property="og:description" content=description.clone()/>
        <Meta property="og:image" content=image.clone()/>
        <Meta property="og:url" content=url.clone()/>
        <Meta property="og:type" content="article"/>
        
        // Twitter Card
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=title/>
        <Meta name="twitter:description" content=description/>
        <Meta name="twitter:image" content=image/>
    }
}
```

---

## File Structure

```
src/
├── api/
│   ├── comments.rs         # Comment API functions
│   └── posts.rs            # Add guest_get_post
├── components/
│   └── view_post/
│       ├── mod.rs          # Module exports
│       ├── post_media.rs   # Image gallery, video, audio
│       ├── post_header.rs  # Author info, follow button
│       ├── post_content.rs # Title, description, tags
│       ├── post_actions.rs # Like, dislike, save, share
│       ├── comments.rs     # Comments list and form
│       ├── share_modal.rs  # Share dialog
│       └── image_modal.rs  # Fullscreen image viewer
├── hooks/
│   └── use_mobile_redirect.rs
├── models/
│   └── comment.rs          # Comment types
├── pages/
│   ├── mod.rs              # Add view_post export
│   └── view_post.rs        # View post page
└── style/
    └── view-post.scss      # View post styles
```

---

## Testing Plan

### Unit Tests

- [ ] Comment model deserialization
- [ ] Guest post API response parsing
- [ ] Time formatting utilities

### Integration Tests

- [ ] Guest can view post without auth
- [ ] Authenticated user can like/dislike/save
- [ ] Comment creation and display
- [ ] Nested replies loading
- [ ] Share link generation

### E2E Tests (Playwright)

- [ ] Navigate to `/post/{id}` as guest
- [ ] Verify post media displays
- [ ] Verify comment section loads
- [ ] Verify "Sign Up" CTA appears for guests
- [ ] Login and verify interactions work
- [ ] Create a comment
- [ ] Like/unlike a comment
- [ ] Share via copy link
- [ ] Mobile viewport deep link behavior

---

## Acceptance Criteria

1. **Guest Mode**
   - [ ] Post loads without authentication
   - [ ] All post content visible (media, title, description, tags)
   - [ ] Comments visible but cannot interact
   - [ ] "View-only mode" banner displayed
   - [ ] "Sign Up" CTA visible and functional

2. **Authenticated Mode**
   - [ ] Full post data with interaction states
   - [ ] Like/dislike/save buttons functional
   - [ ] Comment creation works
   - [ ] Reply to comments works
   - [ ] Like comments works
   - [ ] Report post accessible

3. **Media**
   - [ ] Image gallery with navigation
   - [ ] Video playback with controls
   - [ ] Audio playback with controls
   - [ ] Text posts display correctly

4. **Comments**
   - [ ] Top-level comments load paginated
   - [ ] Nested replies load on demand
   - [ ] New comments appear at top
   - [ ] Like count updates optimistically

5. **Share**
   - [ ] Copy link to clipboard works
   - [ ] WhatsApp link opens correctly
   - [ ] Telegram link opens correctly

6. **SEO**
   - [ ] Dynamic title tag
   - [ ] Open Graph meta tags
   - [ ] Twitter Card meta tags

7. **Responsive**
   - [ ] Desktop layout (two-panel)
   - [ ] Mobile layout (stacked)
   - [ ] Touch-friendly interactions

---

## Dependencies

- Existing: `leptos`, `leptos_router`, `leptos_meta`
- Existing: `serde`, `serde_json`
- Existing: Auth context, GraphQL client
- New: None required

---

## Estimated Timeline

| Phase | Description | Days |
|-------|-------------|------|
| Phase 1 | Models & API Layer | 1 |
| Phase 2 | View Post Page | 1 |
| Phase 3 | Post Media Components | 2 |
| Phase 4 | Comments Components | 2 |
| Phase 5 | Post Actions & Share | 1 |
| Phase 6 | Mobile & SEO | 0.5 |
| Testing | Unit, Integration, E2E | 1.5 |
| **Total** | | **9 days** |

---

## Changelog

### 2026-04-12
- Initial planning document created
- Analyzed legacy implementation
- Defined scope and phases
- Created Rust code structure
- Documented API requirements
