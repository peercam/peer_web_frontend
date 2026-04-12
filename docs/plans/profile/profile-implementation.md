# Profile Implementation Plan

**Feature:** Profile (My Profile + View Profile)  
**Priority:** #5 (after View Post)  
**Status:** 📋 Planning  
**Created:** 2026-04-12

---

## Overview

Implement the user profile pages for the Leptos frontend. This includes:
1. **My Profile** (`/profile`) — The authenticated user's own profile with edit capabilities and post boost features
2. **View Profile** (`/profile/:slug` or `/u/:slug`) — Viewing other users' profiles with follow/block/report actions

Both pages display the user's posts, social statistics, and allow interactions based on the viewer's relationship with the profile owner.

### Goals

1. Full parity with legacy `profile.php` and `view-profile.php` user experience
2. Authenticated-only pages (requires login)
3. Profile header with avatar, username, slug, bio, and social stats
4. User's post feed with infinite scroll, filters, and sorting
5. Social actions: follow/unfollow, block, report
6. Content moderation overlays (hidden/illegal content handling)
7. Relations modal (followers/following/peers lists)
8. Responsive layout matching dashboard pattern
9. Profile boost/ads features (My Profile only)

---

## Scope

### In Scope

- [ ] My Profile page (`/profile` route)
- [ ] View Profile page (`/profile/:slug` or `/u/:slug` route)
- [ ] Profile header component:
  - [ ] Avatar with online status indicator
  - [ ] Username and slug display
  - [ ] Biography (fetched from media server)
  - [ ] Social statistics (posts, followers, following, peers)
- [ ] Post feed with filters and infinite scroll
- [ ] My Profile features:
  - [ ] Edit profile button (→ settings)
  - [ ] Ads dropdown (Boost post, My Ads)
  - [ ] Post selection mode for boosting
- [ ] View Profile features:
  - [ ] Follow/unfollow button with state management
  - [ ] More actions dropdown (report, block)
  - [ ] Mutual follow indicator
- [ ] Relations modal:
  - [ ] Followers tab with infinite scroll
  - [ ] Following tab with infinite scroll
  - [ ] Peers tab (mutual follows, own profile only)
  - [ ] Follow button in user list items
- [ ] Content visibility handling:
  - [ ] Hidden content overlay with "View anyway" option
  - [ ] Illegal content badge (profile removed)
  - [ ] Reported content badge
- [ ] Loading states and skeletons
- [ ] Error states (profile not found, blocked)
- [ ] Responsive layout (desktop/tablet/mobile)

### Out of Scope (Future Work)

- Edit profile form (separate feature: Settings)
- Ads management details (separate feature: My Ads)
- Notification settings
- Chat integration
- Wallet widget

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `profile.php` | My Profile page template |
| `view-profile.php` | View Profile page template |
| `js/profile.js` | My Profile data loading, visibility handling |
| `js/viewprofile.js` | View Profile data, follow actions, visibility |
| `js/list_follow.js` | Relations modal (followers/following/peers) |
| `js/reports/report_user.js` | User reporting functionality |
| `css/profile.css` | Profile-specific styles |
| `template-parts/sidebars/widget-filter.php` | Content type filters |
| `template-parts/sidebars/widget-sort-filter.php` | Sort options |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  HEADER: Logo + "Profile" title                            │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│  (16rem)     │  ┌─────────────────┐    │  (17rem)          │
│              │  │ PROFILE HEADER  │    │                   │
│  - Content   │  │ Avatar │ Info   │    │  - Daily Actions  │
│    Filters   │  │ Bio    │ Stats  │    │  - Main Menu      │
│  - Sort      │  │ [Edit] │ [Ads]  │    │  - New Post       │
│    Options   │  └─────────────────┘    │  - Version        │
│              │                         │                   │
│              │  ┌─────────────────┐    │                   │
│              │  │ POST GRID       │    │                   │
│              │  │ (infinite scroll│    │                   │
│              │  │  with filters)  │    │                   │
│              │  └─────────────────┘    │                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### Profile Header Detail

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                  │
│  ┌─────────┐   Username                    ┌──────────────────┐  │
│  │         │   #12345 (slug)               │ [Edit] / [Follow]│  │
│  │  Avatar │                               │                  │  │
│  │         │   Biography text here...      │ [Ads ▼] / [More▼]│  │
│  └─────────┘                               └──────────────────┘  │
│                                                                  │
│  123 Publications  |  456 Followers  |  789 Following  |  12 Peers
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Mobile Layout

```
┌──────────────────────────┐
│ [←] Profile     [⋮]      │
├──────────────────────────┤
│   ┌─────┐                │
│   │Avatar│               │
│   └─────┘                │
│                          │
│   Username               │
│   #12345                 │
│                          │
│   Biography...           │
│                          │
│   [Edit Profile] / [Follow]
│   [Ads ▼] / [More ▼]     │
├──────────────────────────┤
│ 📝 123  👥 456  ➡️ 789    │
├──────────────────────────┤
│ [Filters collapse toggle]│
├──────────────────────────┤
│ Post Grid                │
│ (2 columns on mobile)    │
│                          │
├──────────────────────────┤
│ 🏠  📷  💬  👤  ⚙️        │
└──────────────────────────┘
```

### Key Features

1. **My Profile (Own Profile)**
   - Uses `getProfile()` query with no userid (defaults to self)
   - Edit button links to `/settings`
   - Ads dropdown:
     - "Boost post" → enables post selection mode
     - "My Ads" → links to `/my-ads`
   - Posts filtered to `userid: currentUser`

2. **View Profile (Other Users)**
   - Uses `getProfile(userid)` query
   - URL param: `?user=<uuid>` (legacy) or route param `/:slug`
   - Follow button with states:
     - "Follow" (not following)
     - "Following" (currently following, click to unfollow)
     - "Follow Back" (they follow you, you don't follow them)
   - More dropdown:
     - "Report profile" → reports user
     - "Block content" → blocks user

3. **Relations Modal**
   - Opens on click of Followers/Following/Peers count
   - Tabs: Followers | Following | Peers (peers only on own profile)
   - User list items with:
     - Avatar, username, slug
     - Follow button
     - Click → navigate to their profile
   - Infinite scroll within modal

4. **Content Visibility States**
   - `NORMAL`: Standard display
   - `HIDDEN`: Shows overlay "Sensitive content" with "View anyway" button
   - `ILLEGAL`: Replaces all profile data with "removed" + illegal badge
   - `hasActiveReports`: Shows reported badge on profile

5. **Biography Loading**
   - Biography is stored as a path to a text file on media server
   - Fetch the file content and display as text
   - Fallback: "Biography not available"

---

## Backend API Reference

### `getProfile` Query

```graphql
query GetProfile($userid: ID, $contentFilterBy: ContentFilterType) {
  getProfile(userid: $userid, contentFilterBy: $contentFilterBy) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    affectedRows {
      id
      username
      status
      slug
      img
      biography
      visibilityStatus
      isHiddenForUsers
      hasActiveReports
      iFollowThisUser             # Do I follow this profile?
      thisUserFollowsMe           # Does this profile follow me?
      isreported                  # Have I reported this user?
      amountposts
      amounttrending
      amountfollowed              # Users this profile follows
      amountfollower              # Followers count
      amountfriends               # Mutual follows (peers)
      amountblocked
      amountreports
    }
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11008` | Profile loaded successfully |
| `21001` | User not found |
| `30201` | Invalid user UUID |
| `60501` | Not authenticated |

### `getUserInfo` Query

```graphql
query GetUserInfo {
  getUserInfo {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    affectedRows {
      userid
      liquidity
      amountposts
      amountreports
      amountblocked
      amountfollower
      amountfollowed
      amountfriends
      invited
      updatedat
      userPreferences {
        contentFilteringSeverityLevel
        onboardingsWereShown
      }
    }
  }
}
```

### `listFollowRelations` Query

```graphql
query ListFollowRelations(
  $userid: ID
  $contentFilterBy: ContentFilterType
  $offset: Int
  $limit: Int
) {
  listFollowRelations(
    userid: $userid
    contentFilterBy: $contentFilterBy
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
      followers {
        userid
        username
        slug
        img
        visibilityStatus
        isHiddenForUsers
        hasActiveReports
        isfollowed              # Do I follow this person?
        isfollowing             # Does this person follow me?
      }
      following {
        userid
        username
        slug
        img
        visibilityStatus
        isHiddenForUsers
        hasActiveReports
        isfollowed
        isfollowing
      }
    }
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11101` | Follow data loaded |
| `21102` | No relations found |
| `60501` | Not authenticated |

### `listFriends` Query (Peers / Mutual Follows)

```graphql
query ListFriends($userid: ID, $contentFilterBy: ContentFilterType, $offset: Int, $limit: Int) {
  listFriends(
    userid: $userid
    contentFilterBy: $contentFilterBy
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
      userid
      img
      username
      slug
      biography
      visibilityStatus
      isHiddenForUsers
      hasActiveReports
      updatedat
    }
  }
}
```

### `toggleUserFollowStatus` Mutation

```graphql
mutation ToggleUserFollowStatus($userid: ID!) {
  toggleUserFollowStatus(userid: $userid) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    isfollowing           # New follow state after toggle
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11104` | Now following user |
| `11103` | Unfollowed user |
| `30201` | Invalid UUID |
| `60501` | Not authenticated |

### `toggleBlockUserStatus` Mutation

```graphql
mutation ToggleBlockUserStatus($userid: ID!) {
  toggleBlockUserStatus(userid: $userid) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11105` | User blocked |
| `11106` | User unblocked |
| `30201` | Invalid UUID |
| `60501` | Not authenticated |

### `reportUser` Mutation

```graphql
mutation ReportUser($userid: ID!) {
  reportUser(userid: $userid) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11012` | User reported |
| `30201` | Invalid UUID |
| `31007` | User not found |
| `31008` | Already reported (duplicate) |
| `31009` | Cannot report yourself |
| `60501` | Not authenticated |

### `listPosts` Query (User's Posts)

```graphql
query ListPosts(
  $userid: ID!
  $filterBy: [PostFilterType!]
  $contentFilterBy: ContentFilterType
  $sortBy: PostSortType
  $offset: Int
  $limit: Int
) {
  listPosts(
    userid: $userid
    filterBy: $filterBy
    contentFilterBy: $contentFilterBy
    sortBy: $sortBy
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

---

## Implementation Plan

### Phase 1: Models & API Layer

#### 1.1 Profile Models (`src/models/profile.rs`)

```rust
use serde::{Deserialize, Serialize};

use crate::models::common::DefaultResponse;

/// User visibility status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentVisibilityStatus {
    #[default]
    Normal,
    Hidden,
    Illegal,
}

/// Full profile data from `getProfile` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub username: String,
    pub status: i32,
    pub slug: i32,
    pub img: Option<String>,
    pub biography: Option<String>,
    #[serde(default)]
    pub visibility_status: ContentVisibilityStatus,
    #[serde(default)]
    pub is_hidden_for_users: bool,
    #[serde(default)]
    pub has_active_reports: bool,
    /// Do I follow this profile?
    #[serde(default, rename = "iFollowThisUser")]
    pub i_follow_this_user: bool,
    /// Does this profile follow me?
    #[serde(default, rename = "thisUserFollowsMe")]
    pub this_user_follows_me: bool,
    /// Have I reported this user?
    #[serde(default)]
    pub isreported: bool,
    #[serde(default)]
    pub amountposts: i32,
    #[serde(default)]
    pub amounttrending: i32,
    #[serde(default)]
    pub amountfollowed: i32,
    #[serde(default)]
    pub amountfollower: i32,
    #[serde(default)]
    pub amountfriends: i32,
    #[serde(default)]
    pub amountblocked: i32,
    #[serde(default)]
    pub amountreports: i32,
}

/// Profile API response wrapper.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
    pub meta: DefaultResponse,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<Profile>,
}

impl ProfileResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// User in a follow list (follower/following).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUser {
    pub userid: String,
    pub username: String,
    pub slug: i32,
    pub img: Option<String>,
    #[serde(default)]
    pub visibility_status: ContentVisibilityStatus,
    #[serde(default)]
    pub is_hidden_for_users: bool,
    #[serde(default)]
    pub has_active_reports: bool,
    /// Do I follow this person?
    #[serde(default)]
    pub isfollowed: bool,
    /// Does this person follow me?
    #[serde(default)]
    pub isfollowing: bool,
}

/// Follow relations response (followers + following).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRelations {
    #[serde(default)]
    pub followers: Vec<ProfileUser>,
    #[serde(default)]
    pub following: Vec<ProfileUser>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRelationsResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<FollowRelations>,
}

/// Basic user info (for friends/peers list).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicUserInfo {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: i32,
    pub biography: Option<String>,
    #[serde(default)]
    pub visibility_status: ContentVisibilityStatus,
    #[serde(default)]
    pub is_hidden_for_users: bool,
    #[serde(default)]
    pub has_active_reports: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendsResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Vec<BasicUserInfo>,
}

/// Toggle follow status response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowStatusResponse {
    pub meta: DefaultResponse,
    /// New follow state after toggle.
    pub isfollowing: bool,
}
```

#### 1.2 Profile API (`src/api/profile.rs`)

```rust
use leptos::prelude::*;

use crate::models::profile::{
    FollowRelationsResponse, FollowStatusResponse, FriendsResponse, Profile, ProfileResponse,
};

/// Fetch a user's profile. If `user_id` is None, fetches current user's profile.
#[server(GetProfile, "/api")]
pub async fn get_profile(user_id: Option<String>) -> Result<Profile, ServerFnError> {
    use crate::api::graphql::{query_with_auth, GET_PROFILE_QUERY};

    #[derive(serde::Serialize)]
    struct Vars {
        userid: Option<String>,
    }

    let vars = Vars { userid: user_id };
    let res: ProfileResponse = query_with_auth(GET_PROFILE_QUERY, vars).await?;

    res.affected_rows
        .ok_or_else(|| ServerFnError::new("Profile not found"))
}

/// Fetch followers and following for a user.
#[server(ListFollowRelations, "/api")]
pub async fn list_follow_relations(
    user_id: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<FollowRelationsResponse, ServerFnError>;

/// Fetch mutual follows (peers/friends).
#[server(ListFriends, "/api")]
pub async fn list_friends(
    user_id: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<FriendsResponse, ServerFnError>;

/// Toggle follow status for a user.
#[server(ToggleFollow, "/api")]
pub async fn toggle_follow(user_id: String) -> Result<FollowStatusResponse, ServerFnError>;

/// Block/unblock a user.
#[server(ToggleBlock, "/api")]
pub async fn toggle_block(user_id: String) -> Result<(), ServerFnError>;

/// Report a user.
#[server(ReportUser, "/api")]
pub async fn report_user(user_id: String) -> Result<(), ServerFnError>;

/// Fetch biography text from media server.
#[server(FetchBiography, "/api")]
pub async fn fetch_biography(bio_path: String) -> Result<String, ServerFnError> {
    let media_host = std::env::var("MEDIA_HOST")
        .unwrap_or_else(|_| "https://media.getpeer.eu".to_string());
    
    let url = format!("{}/{}", media_host, bio_path);
    let response = reqwest::get(&url).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ServerFnError::new("Biography not available"));
    }
    
    response.text().await.map_err(|e| ServerFnError::new(e.to_string()))
}
```

#### 1.3 GraphQL Queries (`src/api/graphql.rs` additions)

```rust
pub const GET_PROFILE_QUERY: &str = r#"
    query GetProfile($userid: ID, $contentFilterBy: ContentFilterType) {
        getProfile(userid: $userid, contentFilterBy: $contentFilterBy) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            affectedRows {
                id
                username
                status
                slug
                img
                biography
                visibilityStatus
                isHiddenForUsers
                hasActiveReports
                iFollowThisUser
                thisUserFollowsMe
                isreported
                amountposts
                amounttrending
                amountfollowed
                amountfollower
                amountfriends
                amountblocked
                amountreports
            }
        }
    }
"#;

pub const LIST_FOLLOW_RELATIONS_QUERY: &str = r#"
    query ListFollowRelations(
        $userid: ID
        $contentFilterBy: ContentFilterType
        $offset: Int
        $limit: Int
    ) {
        listFollowRelations(
            userid: $userid
            contentFilterBy: $contentFilterBy
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
                followers {
                    userid
                    username
                    slug
                    img
                    visibilityStatus
                    isHiddenForUsers
                    hasActiveReports
                    isfollowed
                    isfollowing
                }
                following {
                    userid
                    username
                    slug
                    img
                    visibilityStatus
                    isHiddenForUsers
                    hasActiveReports
                    isfollowed
                    isfollowing
                }
            }
        }
    }
"#;

pub const LIST_FRIENDS_QUERY: &str = r#"
    query ListFriends(
        $userid: ID
        $contentFilterBy: ContentFilterType
        $offset: Int
        $limit: Int
    ) {
        listFriends(
            userid: $userid
            contentFilterBy: $contentFilterBy
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
                userid
                img
                username
                slug
                biography
                visibilityStatus
                isHiddenForUsers
                hasActiveReports
            }
        }
    }
"#;

pub const TOGGLE_FOLLOW_MUTATION: &str = r#"
    mutation ToggleUserFollowStatus($userid: ID!) {
        toggleUserFollowStatus(userid: $userid) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            isfollowing
        }
    }
"#;

pub const TOGGLE_BLOCK_MUTATION: &str = r#"
    mutation ToggleBlockUserStatus($userid: ID!) {
        toggleBlockUserStatus(userid: $userid) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
        }
    }
"#;

pub const REPORT_USER_MUTATION: &str = r#"
    mutation ReportUser($userid: ID!) {
        reportUser(userid: $userid) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
        }
    }
"#;
```

### Phase 2: Profile Components

#### 2.1 Profile Header (`src/components/profile/header.rs`)

```rust
use leptos::prelude::*;

use crate::models::profile::{ContentVisibilityStatus, Profile};

/// Profile header with avatar, info, and actions.
#[component]
pub fn ProfileHeader(
    profile: Profile,
    /// Whether this is the current user's own profile.
    is_own_profile: bool,
    /// Callback when follow button is clicked.
    #[prop(optional)]
    on_follow: Option<Callback<()>>,
) -> impl IntoView {
    let is_hidden = profile.visibility_status == ContentVisibilityStatus::Hidden
        || profile.is_hidden_for_users;
    let is_illegal = profile.visibility_status == ContentVisibilityStatus::Illegal;

    view! {
        <div class="profile-header" class:illegal=is_illegal class:hidden=is_hidden>
            <ProfileAvatar
                src=profile.img.clone()
                is_illegal=is_illegal
            />
            <ProfileInfo profile=profile.clone() is_illegal=is_illegal/>
            <ProfileActions
                profile=profile
                is_own_profile=is_own_profile
                on_follow=on_follow
            />
        </div>
    }
}

/// Profile avatar with visibility handling.
#[component]
fn ProfileAvatar(src: Option<String>, is_illegal: bool) -> impl IntoView {
    let default_avatar = "/svg/noname.svg".to_string();
    
    view! {
        <div class="profile-picture">
            <div class="crop-container">
                <span class="online-status"></span>
                {if is_illegal {
                    view! {
                        <div class="illegal-profile-frame">
                            <i class="peer-icon peer-icon-illegal"></i>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <img
                            class="profile-img"
                            src=src.unwrap_or(default_avatar)
                            alt="Profile Picture"
                            on:error=|ev| {
                                let target = ev.target().unwrap();
                                let img: web_sys::HtmlImageElement = target.unchecked_into();
                                img.set_src("/svg/noname.svg");
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

/// Profile info section (username, slug, bio, stats).
#[component]
fn ProfileInfo(profile: Profile, is_illegal: bool) -> impl IntoView {
    let bio_signal = RwSignal::new(None::<String>);
    
    // Fetch biography from media server
    if let Some(bio_path) = profile.biography.clone() {
        spawn_local(async move {
            if let Ok(text) = fetch_biography(bio_path).await {
                bio_signal.set(Some(text));
            }
        });
    }
    
    let username = if is_illegal { "removed".to_string() } else { profile.username.clone() };
    let slug = if is_illegal { "removed".to_string() } else { format!("#{}", profile.slug) };

    view! {
        <div class="profile-info">
            <h2 class="profile-title">
                <span class="username">{username}</span>
                <span class="slug">{slug}</span>
            </h2>
            
            <div class="profile-description">
                {move || bio_signal.get().unwrap_or_else(|| "Biography not available".into())}
            </div>
            
            <ProfileStats profile=profile.clone()/>
        </div>
    }
}

/// Profile statistics (posts, followers, following, peers).
#[component]
fn ProfileStats(profile: Profile) -> impl IntoView {
    view! {
        <div class="profile-stats">
            <span class="stat post-count">
                <em>{profile.amountposts}</em>
                " Publications"
            </span>
            <span class="stat followers-count clickable" on:click=|_| { /* open modal */ }>
                <em>{profile.amountfollower}</em>
                " Followers"
            </span>
            <span class="stat following-count clickable" on:click=|_| { /* open modal */ }>
                <em>{profile.amountfollowed}</em>
                " Following"
            </span>
            <span class="stat peers-count">
                <em>{profile.amountfriends}</em>
                " Peers"
            </span>
        </div>
    }
}

/// Profile action buttons (Edit/Follow, Ads/More dropdown).
#[component]
fn ProfileActions(
    profile: Profile,
    is_own_profile: bool,
    #[prop(optional)]
    on_follow: Option<Callback<()>>,
) -> impl IntoView {
    if is_own_profile {
        view! { <OwnProfileActions/> }.into_any()
    } else {
        view! {
            <ViewProfileActions
                profile=profile
                on_follow=on_follow
            />
        }.into_any()
    }
}

/// Actions for viewing your own profile.
#[component]
fn OwnProfileActions() -> impl IntoView {
    let ads_expanded = RwSignal::new(false);
    
    view! {
        <div class="profile-edit-box">
            <a href="/settings" class="button btn-white edit-profile">
                <i class="peer-icon peer-icon-edit-pencil"></i>
                "Edit"
            </a>
            
            <div class="ads-container-wrap">
                <button
                    class="button ads-button"
                    aria-expanded=move || ads_expanded.get()
                    on:click=move |_| ads_expanded.update(|v| *v = !*v)
                >
                    "Ads"
                </button>
                
                <div class="ads-dropdown" class:open=move || ads_expanded.get()>
                    <button class="button btn-blue boost-posts">
                        <i class="peer-icon peer-icon-megaphone"></i>
                        "Boost post"
                    </button>
                    <a href="/my-ads" class="button btn-white my-ads">
                        <i class="peer-icon peer-icon-ad"></i>
                        "My Ads"
                    </a>
                </div>
            </div>
        </div>
    }
}

/// Actions when viewing another user's profile.
#[component]
fn ViewProfileActions(
    profile: Profile,
    #[prop(optional)]
    on_follow: Option<Callback<()>>,
) -> impl IntoView {
    let is_following = RwSignal::new(profile.i_follow_this_user);
    let they_follow_me = profile.this_user_follows_me;
    let more_expanded = RwSignal::new(false);
    let is_reported = RwSignal::new(profile.isreported);
    
    let follow_text = move || {
        if is_following.get() {
            "Following"
        } else if they_follow_me {
            "Follow Back"
        } else {
            "Follow"
        }
    };
    
    view! {
        <div class="profile-edit-box">
            <button
                class="button btn-transparent follow-button"
                class:following=move || is_following.get()
                on:click=move |_| {
                    if let Some(cb) = on_follow {
                        cb.call(());
                    }
                }
            >
                {follow_text}
            </button>
            
            <div class="more-actions-wrap">
                <button
                    class="button more-actions"
                    aria-expanded=move || more_expanded.get()
                    on:click=move |_| more_expanded.update(|v| *v = !*v)
                >
                    "More"
                </button>
                
                <div class="more-dropdown" class:open=move || more_expanded.get()>
                    <button
                        class="report-profile"
                        disabled=move || is_reported.get()
                        on:click=move |_| { /* handle report */ }
                    >
                        {move || if is_reported.get() { "Reported" } else { "Report profile" }}
                    </button>
                    <button class="block-content" on:click=move |_| { /* handle block */ }>
                        "Block content"
                    </button>
                </div>
            </div>
        </div>
    }
}
```

#### 2.2 Relations Modal (`src/components/profile/relations_modal.rs`)

```rust
use leptos::prelude::*;

use crate::api::profile::{list_follow_relations, list_friends};
use crate::models::profile::ProfileUser;

/// Tab type for the relations modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationsTab {
    Followers,
    Following,
    Peers,
}

/// Modal showing followers, following, and peers.
#[component]
pub fn RelationsModal(
    /// User ID to show relations for.
    user_id: String,
    /// Whether viewing own profile (shows Peers tab).
    is_own_profile: bool,
    /// Default tab to open.
    default_tab: RelationsTab,
    /// Called when modal should close.
    on_close: Callback<()>,
) -> impl IntoView {
    let active_tab = RwSignal::new(default_tab);
    
    view! {
        <div class="modal-overlay" on:click=move |e| {
            if e.target() == e.current_target() {
                on_close.call(());
            }
        }>
            <div class="modal-content relations-modal">
                <button class="modal-close" on:click=move |_| on_close.call(())>
                    "×"
                </button>
                
                <div class="tabs">
                    <TabButton
                        label="Followers"
                        tab=RelationsTab::Followers
                        active_tab
                    />
                    <TabButton
                        label="Following"
                        tab=RelationsTab::Following
                        active_tab
                    />
                    {if is_own_profile {
                        Some(view! {
                            <TabButton
                                label="Peers"
                                tab=RelationsTab::Peers
                                active_tab
                            />
                        })
                    } else {
                        None
                    }}
                </div>
                
                <div class="modal-body">
                    <RelationsTabContent
                        user_id=user_id.clone()
                        tab=active_tab.get()
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn TabButton(
    label: &'static str,
    tab: RelationsTab,
    active_tab: RwSignal<RelationsTab>,
) -> impl IntoView {
    view! {
        <button
            class="tab-btn"
            class:active=move || active_tab.get() == tab
            on:click=move |_| active_tab.set(tab)
        >
            {label}
        </button>
    }
}

#[component]
fn RelationsTabContent(user_id: String, tab: RelationsTab) -> impl IntoView {
    // Implementation with infinite scroll loading
    // ...
}

#[component]
fn UserListItem(user: ProfileUser) -> impl IntoView {
    let is_following = RwSignal::new(user.isfollowed);
    
    view! {
        <div class="user-list-item">
            <a href=format!("/profile/{}", user.slug) class="user-info">
                <img
                    class="avatar"
                    src=user.img.unwrap_or_else(|| "/svg/noname.svg".into())
                    alt=&user.username
                />
                <div class="user-details">
                    <span class="username">{&user.username}</span>
                    <span class="slug">{"#"}{user.slug}</span>
                </div>
            </a>
            <button
                class="button btn-transparent follow-btn"
                class:following=move || is_following.get()
                on:click=move |_| { /* toggle follow */ }
            >
                {move || if is_following.get() { "Following" } else { "Follow" }}
            </button>
        </div>
    }
}
```

### Phase 3: Profile Pages

#### 3.1 My Profile Page (`src/pages/profile.rs`)

```rust
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_navigate;

use crate::api::profile::get_profile;
use crate::components::profile::{ProfileHeader, RelationsModal};
use crate::components::posts::PostList;
use crate::state::auth::use_auth;

/// My Profile page.
#[component]
pub fn MyProfilePage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    
    // Redirect if not authenticated
    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            navigate("/login", Default::default());
        }
    });
    
    let profile_resource = Resource::new(
        || (),
        |_| async { get_profile(None).await },
    );
    
    view! {
        <Title text="Profile - Peer Network"/>
        
        <div class="site-layout my-profile-page">
            <header class="site-header">
                <img class="logo" src="/svg/dashboard-profile.svg" alt="Peer Network"/>
                <h1>"Profile"</h1>
            </header>
            
            <aside class="left-sidebar">
                // Filters sidebar (reuse from dashboard)
                <FiltersSidebar/>
            </aside>
            
            <main class="site-main profile-main">
                <Suspense fallback=move || view! { <ProfileSkeleton/> }>
                    {move || profile_resource.get().map(|result| {
                        match result {
                            Ok(profile) => view! {
                                <ProfileHeader
                                    profile=profile.clone()
                                    is_own_profile=true
                                />
                                <PostList user_id=Some(profile.id.clone())/>
                            }.into_any(),
                            Err(_) => view! { <ProfileError/> }.into_any(),
                        }
                    })}
                </Suspense>
            </main>
            
            <aside class="right-sidebar">
                // Menu sidebar
                <MenuSidebar/>
            </aside>
            
            <Footer/>
        </div>
    }
}
```

#### 3.2 View Profile Page (`src/pages/view_profile.rs`)

```rust
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::api::profile::{get_profile, toggle_follow};
use crate::components::profile::ProfileHeader;
use crate::components::posts::PostList;
use crate::state::auth::use_auth;

/// View another user's profile page.
#[component]
pub fn ViewProfilePage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let params = use_params_map();
    
    // Redirect if not authenticated
    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            navigate("/login", Default::default());
        }
    });
    
    let user_slug_or_id = move || {
        params.get()
            .get("slug")
            .map(|s| s.to_string())
            .or_else(|| {
                // Support legacy ?user= param
                #[cfg(feature = "hydrate")]
                {
                    web_sys::window()
                        .and_then(|w| w.location().search().ok())
                        .and_then(|s| {
                            url::form_urlencoded::parse(s.trim_start_matches('?').as_bytes())
                                .find(|(k, _)| k == "user")
                                .map(|(_, v)| v.into_owned())
                        })
                }
                #[cfg(not(feature = "hydrate"))]
                None
            })
    };
    
    let profile_resource = Resource::new(
        user_slug_or_id,
        |id| async move {
            match id {
                Some(id) => get_profile(Some(id)).await,
                None => Err(ServerFnError::new("No user specified")),
            }
        },
    );
    
    let handle_follow = move |_| {
        spawn_local(async move {
            if let Some(Ok(profile)) = profile_resource.get() {
                let _ = toggle_follow(profile.id.clone()).await;
                // Refetch profile to update state
                profile_resource.refetch();
            }
        });
    };
    
    view! {
        <div class="site-layout view-profile-page">
            <header class="site-header">
                <img class="logo" src="/svg/Home.svg" alt="Peer Network"/>
                <h1>"Profile"</h1>
            </header>
            
            <aside class="left-sidebar">
                <FiltersSidebar/>
            </aside>
            
            <main class="site-main profile-main">
                <Suspense fallback=move || view! { <ProfileSkeleton/> }>
                    {move || profile_resource.get().map(|result| {
                        match result {
                            Ok(profile) => {
                                let title = format!("{} - Peer Network", profile.username);
                                view! {
                                    <Title text=title/>
                                    <ProfileHeader
                                        profile=profile.clone()
                                        is_own_profile=false
                                        on_follow=handle_follow.into()
                                    />
                                    <PostList user_id=Some(profile.id.clone())/>
                                }.into_any()
                            },
                            Err(_) => view! {
                                <Title text="Profile Not Found - Peer Network"/>
                                <ProfileNotFound/>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </main>
            
            <aside class="right-sidebar">
                <ProfileWidget/>
                <MenuSidebar/>
            </aside>
            
            <Footer/>
        </div>
    }
}
```

### Phase 4: Routing & Integration

#### 4.1 Router Updates (`src/app.rs`)

```rust
// Add to route definitions:
<Route path="/profile" view=MyProfilePage/>
<Route path="/profile/:slug" view=ViewProfilePage/>
<Route path="/u/:slug" view=ViewProfilePage/>  // Alternative URL format
```

#### 4.2 Module Structure

```
src/
├── api/
│   ├── mod.rs
│   └── profile.rs          # Profile API functions
├── components/
│   ├── mod.rs
│   └── profile/
│       ├── mod.rs
│       ├── header.rs       # ProfileHeader component
│       ├── actions.rs      # Profile action buttons
│       ├── stats.rs        # Profile statistics
│       ├── relations_modal.rs  # Followers/following modal
│       ├── user_list.rs    # User list item component
│       └── visibility.rs   # Content visibility overlays
├── models/
│   ├── mod.rs
│   └── profile.rs          # Profile types
└── pages/
    ├── mod.rs
    ├── profile.rs          # My Profile page
    └── view_profile.rs     # View Profile page
```

### Phase 5: Styling

#### 5.1 Profile Styles (`style/profile.scss`)

- Port styles from legacy `css/profile.css`
- Use SCSS variables for consistency
- Implement responsive breakpoints
- Handle visibility states (hidden, illegal, reported)

---

## Testing Strategy

### Unit Tests

- [ ] Profile model serialization/deserialization
- [ ] Visibility status enum handling
- [ ] Follow button state logic

### Integration Tests

- [ ] Profile API calls with mock backend
- [ ] Biography fetching
- [ ] Follow/unfollow mutations
- [ ] Block/report mutations

### E2E Tests (Playwright)

- [ ] Navigate to own profile, verify data displayed
- [ ] Navigate to other user's profile
- [ ] Click stats to open relations modal
- [ ] Follow/unfollow user
- [ ] Report user (verify button state change)
- [ ] Hidden content overlay → "View anyway" button
- [ ] Illegal profile displays "removed" status
- [ ] Infinite scroll on post list
- [ ] Filter and sort posts
- [ ] Mobile responsive layout

---

## Accessibility

- [ ] ARIA labels for interactive elements
- [ ] Keyboard navigation in relations modal
- [ ] Focus management when modal opens/closes
- [ ] Screen reader announcements for state changes
- [ ] Alt text for avatars
- [ ] Skip links for navigation

---

## Dependencies

### Requires (from other features)

- `PostList` component (from Dashboard)
- `FiltersSidebar` component (from Dashboard)
- Auth context and guards
- Toast notifications
- GraphQL client infrastructure

### Provides (for other features)

- `ProfileUser` type (shared across features)
- `toggle_follow` API (used in comments, post cards)
- Profile header components (reusable in other contexts)

---

## Migration Notes

### URL Compatibility

| Legacy URL | New URL | Notes |
|------------|---------|-------|
| `profile.php` | `/profile` | Own profile |
| `view-profile.php?user=<uuid>` | `/profile/:slug` | View by slug preferred |
| — | `/u/:slug` | Short URL alternative |

### Feature Flags

Consider feature flags for:
- Ads/boost functionality (may be deferred)
- Report functionality (depends on moderation system)

---

## Changelog

### 2026-04-12

- Initial planning document created
- Defined scope, layout, and API requirements
- Outlined implementation phases
- Added model and component specifications
