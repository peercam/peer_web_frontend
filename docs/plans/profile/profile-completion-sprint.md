# Profile — Completion Sprint Plan

**Feature:** Profile (My Profile + View Profile + Edit Profile)  
**Priority:** Next after View Post completion sprint  
**Status:** ✅ Tasks 1–8 Implemented + Code Quality Pass  
**Created:** 2026-04-16  
**Updated:** 2026-04-16

---

## Summary

My Profile and View Profile are ~85% implemented. Edit Profile (❌ Not Started) is functionally identical to the existing Settings page — the legacy `edit_profile.php` loads the same settings menu, editProfile, notification, content, and preferences panels as `profileSettings.php`. This sprint closes all three features to ✅ Implemented by addressing the documented gaps and adding a redirect route.

**Convergence impact:** 3 features move to ✅ (Edit Profile, My Profile, View Profile) — bumps overall convergence from ~64% to ~79%.

---

## Code Audit

### What Exists ✅

| Layer | File | Lines | Status |
|-------|------|-------|--------|
| **Page — My Profile** | `src/pages/profile.rs` | 238 | ✅ Profile header, post list, auth guard, right sidebar, mobile footer |
| **Page — View Profile** | `src/pages/view_profile.rs` | 274 | ✅ Slug route, profile fetch, follow actions, not-found/error states |
| **Page — Settings** | `src/pages/settings.rs` | 142 | ✅ Tabbed settings: profile, notifications, preferences, content (covers Edit Profile) |
| **Component — ProfileHeader** | `src/components/profile/header.rs` | 262 | ✅ Avatar, username, slug, bio, stats, online indicator |
| **Component — OwnProfileActions** | `src/components/profile/actions.rs` | 240 | ✅ Edit button, Ads dropdown (boost/my-ads) |
| **Component — ViewProfileActions** | `src/components/profile/actions.rs` | — | ✅ Follow/unfollow, block, report, more-actions dropdown |
| **Component — ProfileStats** | `src/components/profile/stats.rs` | 130 | ✅ Posts, followers, following, peers — clickable for relations modal |
| **Component — RelationsModal** | `src/components/profile/relations_modal.rs` | 271 | ✅ Followers/following/peers tabs with batch loading |
| **Component — Visibility** | `src/components/profile/visibility.rs` | 54 | ✅ Hidden overlay, illegal badge, reported badge |
| **Component — PostCard** | `src/components/posts/post_card.rs` | 304 | ✅ Like/dislike/save, view tracking, content type display |
| **Component — PostList** | `src/components/posts/post_list.rs` | 209 | ✅ Dashboard infinite scroll with IntersectionObserver |
| **Component — Filters** | `src/components/filters/` | 246 | ✅ ContentFilter, FeedFilter, SortFilter (dashboard-wired) |
| **State — FilterState** | `src/state/filters.rs` | — | ✅ localStorage-persisted, reactive, versioned |
| **API — Profile** | `src/api/profile.rs` | — | ✅ getProfile, listUserPosts, toggleFollow, listFollowRelations, listFriends, fetchBiography, blockUser, reportUser |
| **API — Posts** | `src/api/posts.rs` | 409 | ✅ listPosts, listAdPosts, postAction, createPost, searchTags |
| **API — Settings** | `src/api/settings.rs` | 282 | ✅ updateBio, updateProfileImage, updateUsername, updatePassword, updateEmail, updateContentPreferences, deleteAccount |
| **Mock Backend — Users** | `packages/mock_backend/src/schema/query/users.rs` | — | ✅ getProfile, listUsersV2, getUserInfo, listFollowRelations, listFriends |
| **Mock Backend — Profile mutations** | `packages/mock_backend/src/schema/mutation/profile.rs` | — | ✅ toggleUserFollowStatus, blockUser, reportUser, updateBio, updateProfileImage, updateUsername, updatePassword, updateEmail, deleteAccount |
| **Mock Backend — Posts** | `packages/mock_backend/src/schema/query/posts.rs` | — | ✅ listPosts with pagination, filters, sorting |
| **SCSS** | `style/profile.scss` | — | ✅ Desktop layout, responsive, header, posts grid |

### What Remains 🔲

| # | Task | File(s) | Effort | Priority | Status |
|---|------|---------|--------|----------|--------|
| 1 | **Add `/edit-profile` redirect route** | `src/app.rs` | S | P0 | ✅ Done |
| 2 | **Infinite scroll for profile post lists** | `src/pages/profile.rs`, `src/pages/view_profile.rs` | M | P0 | ✅ Done |
| 3 | **Wire filter sidebar to profile pages** | `src/pages/profile.rs`, `src/pages/view_profile.rs` | M | P1 | ✅ Done |
| 4 | **Legacy `?user=<uuid>` query param support** | `src/pages/profile.rs` | S | P1 | ✅ Done |
| 5 | **Wire ProfileWidget to auth context** | `src/components/widgets/` | S | P1 | ✅ Done |
| 6 | **Infinite scroll inside RelationsModal** | `src/components/profile/relations_modal.rs` | M | P2 | ✅ Done |
| 7 | **Deactivate/Delete account UI** | `src/components/settings/` | M | P2 | ✅ Done |
| 8 | **Settings: bio + image save fix** | `src/components/settings/profile.rs` | S | P3 | ✅ Done |
| 9 | **E2E tests (Playwright)** | `end2end/` | L | P2 | ❌ Not Started |

**Legend:** S = Small (< 1 hour), M = Medium (1–3 hours), L = Large (3+ hours)

---

## Task Details

### Task 1: Add `/edit-profile` Redirect Route

**Problem:** Legacy users and bookmarks may navigate to `/edit-profile` or `/edit_profile`. The feature convergence tracker lists Edit Profile as ❌ Not Started, but functionally it is the Settings page.

**Solution:** Add a redirect component that navigates to `/settings`.

**File:** `src/app.rs`

```rust
// Add route alongside existing ones:
<Route path=StaticSegment("edit-profile") view=EditProfileRedirect/>
<Route path=StaticSegment("edit_profile") view=EditProfileRedirect/>
```

```rust
/// Redirect legacy edit-profile URLs to the settings page.
#[component]
fn EditProfileRedirect() -> impl IntoView {
    use leptos_router::hooks::use_navigate;

    let navigate = use_navigate();
    Effect::new(move |_| {
        navigate("/settings", Default::default());
    });

    view! {}
}
```

**Tests:** Verify `/edit-profile` redirects to `/settings` in E2E.

---

### Task 2: Infinite Scroll for Profile Post Lists

**Problem:** Both `MyProfilePage` and `ViewProfilePage` load a single batch of 20 posts and stop. The legacy pages use IntersectionObserver to paginate endlessly.

**Solution:** Extract the infinite scroll pattern already used in `PostList` (dashboard) into the profile post lists. Both `ProfilePostList` and `UserPostList` need the same treatment.

**Files:** `src/pages/profile.rs`, `src/pages/view_profile.rs`

**Approach:**

1. Add `offset`, `is_loading`, `has_more` signals to both post list components
2. Add a `loader_ref` div and set up IntersectionObserver (same pattern as `src/components/posts/post_list.rs` lines 130–170)
3. Append new posts to the reactive list on each load
4. Wire `user_id` as a dependency so changing users resets the feed

**Profile post list changes (both pages follow the same pattern):**

```rust
#[component]
fn ProfilePostList(user_id: String) -> impl IntoView {
    let feed = RwSignal::new(Vec::<FeedItem>::new());
    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let loader_ref = NodeRef::<leptos::html::Div>::new();

    let uid = user_id.clone();
    let load_posts = move || {
        if is_loading.get() || !has_more.get() {
            return;
        }
        is_loading.set(true);
        let user_id = uid.clone();
        let current_offset = offset.get();

        spawn_local(async move {
            match list_user_posts(
                user_id, vec![], None,
                PostSortType::Newest,
                current_offset, 20,
            ).await {
                Ok(response) => {
                    let new_posts = response.affected_rows;
                    let has_new = !new_posts.is_empty();
                    if has_new {
                        feed.update(|f| {
                            for post in new_posts {
                                f.push(FeedItem::Post(post));
                            }
                        });
                        offset.update(|o| *o += 20);
                    }
                    has_more.set(has_new && response.counter > current_offset + 20);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to load posts: {:?}", e);
                }
            }
            is_loading.set(false);
        });
    };

    // IntersectionObserver setup (same as post_list.rs)
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        // ... observer creation (identical pattern to PostList)
    });

    // Initial load
    Effect::new(move |_| {
        if feed.get().is_empty() && !is_loading.get() {
            load_posts();
        }
    });

    view! {
        <div class="profile-posts">
            <h3 class="posts-heading">"Posts"</h3>
            // ... For loop + loader div + empty state
        </div>
    }
}
```

**Refactored:** The IntersectionObserver pattern was extracted into a shared `use_infinite_scroll` hook (`src/hooks/use_infinite_scroll.rs`) and applied to all 6 scroll sites: PostList (dashboard), ProfilePostList, UserPostList, RelationsModal, AdList, and TransactionHistory. This eliminated ~180 lines of duplicated code and fixed a `callback.forget()` memory leak — the hook properly cleans up both the observer and the JS closure on unmount.

---

### Task 3: Wire Filter Sidebar to Profile Pages

**Problem:** The left sidebar in both profile pages is empty (`<aside class="site_sidebar left-sidebar"></aside>`). The legacy pages include content-type and sort filters.

**Solution:** Reuse the filter components already built for the dashboard. Provide filter context on the profile pages and wire it to the post list resource dependency.

**Files:** `src/pages/profile.rs`, `src/pages/view_profile.rs`

**Approach:**

1. Call `provide_filter_context()` at the top of each profile page (same as `DashboardPage`)
2. Replace the empty `<aside>` with `<LeftSidebar/>` from `src/components/dashboard/left_sidebar.rs` (or create a shared `FilterSidebar` component if dashboard-specific references are an issue)
3. Wire `filters.version` and `filters.get_filter_by()` into the profile post list resource so filter changes trigger a re-fetch
4. Pass sort, title, and tag from filter state into `list_user_posts`

**Note:** The `FeedFilter` (all/followers/following) doesn't apply to a single user's posts. Either:
- (a) Omit it from the profile sidebar (only show content-type + sort), or
- (b) Show it but ignore it in the query

Option (a) is cleaner. Create a `<ProfileFilterSidebar>` variant that includes only `ContentFilter` and `SortFilter`.

```rust
#[component]
pub fn ProfileFilterSidebar() -> impl IntoView {
    let filters = use_filter_state();

    view! {
        <aside
            class="left-sidebar left-sidebar-profile"
            class:collapsed=move || filters.is_collapsed.get()
        >
            <div class="inner-scroll for-filters">
                <div class="inner-scroll-filters">
                    <ContentFilter/>
                    <SortFilter/>
                </div>
                <CollapseButton/>
            </div>
        </aside>
    }
}
```

---

### Task 4: Legacy `?user=<uuid>` Query Param Support

**Problem:** The legacy `profile.php` navigated via `profile.php?user=<uuid>` for viewing other users. Old links or cached pages may still use this pattern. The Leptos app has `/profile/:slug` but ignores query params on `/profile`.

**Solution:** In `MyProfilePage`, read the `?user=` query param. If present, redirect to `/profile/<slug>` (or fetch by UUID path `/u/<uuid>`).

**File:** `src/pages/profile.rs`

```rust
// At the top of MyProfilePage or ProfileMainContent:
let params = use_query_map();

Effect::new(move |_| {
    if let Some(user_id) = params.get().get("user") {
        // Redirect to the view-profile route
        let navigate = use_navigate();
        navigate(&format!("/u/{}", user_id), Default::default());
    }
});
```

**Note:** The backend `getProfile(userid:)` accepts a UUID, and the router has `/u/:slug` which could accept UUIDs. Verify the ViewProfilePage resolves both slugs and UUIDs — if not, the API layer already supports UUID lookup via `get_profile(None, Some(userid))`.

---

### Task 5: Wire ProfileWidget to Auth Context

**Problem:** The `ProfileWidget` in the dashboard right sidebar is a placeholder not connected to auth context. It should show the logged-in user's avatar, username, and stats.

**File:** `src/components/widgets/`

**Solution:** Use `use_auth_context()` to get the current user info, then display it in the widget. If the widget doesn't have profile details (just auth info like user ID), make a lightweight profile fetch or cache the profile in auth context at login time.

```rust
#[component]
pub fn ProfileWidget() -> impl IntoView {
    let auth = use_auth_context();

    let profile = Resource::new(
        move || auth.is_authenticated.get(),
        |is_auth| async move {
            if is_auth {
                get_profile(None, None).await.ok()
            } else {
                None
            }
        },
    );

    view! {
        <div class="widget profile-widget">
            <Suspense fallback=|| view! { <WidgetSkeleton/> }>
                {move || profile.get().flatten().map(|p| view! {
                    <a href="/profile" class="widget-profile-link">
                        <img src=p.avatar_url() alt="Profile" class="widget-avatar"/>
                        <span class="widget-username">{p.username}</span>
                    </a>
                })}
            </Suspense>
        </div>
    }
}
```

---

### Task 6: Infinite Scroll Inside RelationsModal

**Problem:** The followers/following/peers tabs load a single batch (e.g., 20 items). Users with many followers cannot view them all.

**File:** `src/components/profile/relations_modal.rs`

**Solution:** Add offset tracking and a "Load more" button (or IntersectionObserver on an in-modal sentinel). The modal container has constrained height, so a sentinel at the bottom of the scrollable area works.

**Approach:**

1. Add `offset` and `has_more` signals per tab
2. Append results on each load
3. Place a loader sentinel at the bottom of the user list inside the scrollable area
4. Each tab resets when switched to

**Effort:** Medium — the data flow is straightforward but three tabs need it.

---

### Task 7: Deactivate/Delete Account UI

**Problem:** The `delete_account` server function exists in `src/api/settings.rs` and the menu item exists, but clicking "Deactivate Profile" is a no-op.

**File:** `src/components/settings/` (new component or extend existing)

**Solution:** Create a `DeactivateAccountPanel` similar to `ChangePasswordPanel`:

1. Warning text explaining the action is permanent
2. Password confirmation input
3. Confirmation modal ("Are you sure?")
4. Call `delete_account(password)` on confirm
5. On success, clear auth context, redirect to `/login`
6. On failure, show error toast

```rust
#[component]
fn DeactivateAccountPanel(on_back: impl Fn() + 'static) -> impl IntoView {
    let password = RwSignal::new(String::new());
    let show_confirm = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let toast = use_toast();
    let auth = use_auth_context();

    let on_confirm = move |_| {
        is_deleting.set(true);
        let pw = password.get();
        spawn_local(async move {
            match delete_account(pw).await {
                Ok(_) => {
                    auth.logout();
                    // Navigate to login handled by auth guard
                }
                Err(e) => {
                    toast.show("Failed to delete account", ToastType::Error);
                    is_deleting.set(false);
                }
            }
        });
    };

    view! {
        <div class="deactivate-panel">
            <h3>"Deactivate Profile"</h3>
            <p class="warning-text">
                "This action is permanent. Your account will be deactivated and cannot be recovered."
            </p>
            <div class="input-field">
                <label>"Confirm your password"</label>
                <input
                    type="password"
                    prop:value=move || password.get()
                    on:input=move |ev| password.set(event_target_value(&ev))
                    placeholder="Enter your password"
                />
            </div>
            <button
                class="btn-danger full-width-btn"
                on:click=move |_| show_confirm.set(true)
                disabled=move || password.get().is_empty()
            >
                "Deactivate my account"
            </button>
            // Confirmation modal overlay...
        </div>
    }
}
```

**Security:** Password confirmation prevents accidental or CSRF-driven deletion.

---

### Task 8: Settings — Bio + Image Save Fix

**Problem:** When the user saves profile changes, the bio update and image update were run via two parallel `spawn_local` tasks with `RwSignal`-based completion tracking. This had a subtle race condition where the "no image" fast path could fire the completion check before the bio task resolved.

**File:** `src/components/settings/profile.rs`

**Solution:** Replaced the dual-`spawn_local` race with a single `spawn_local` that awaits both operations sequentially. The UI benefit of true parallelism for two small HTTP calls is negligible, and this eliminates the race entirely without adding a `futures` crate dependency.

```rust
spawn_local(async move {
    let img_result = match img {
        Some(img_data) => update_profile_image(img_data).await,
        None => Ok(()),
    };
    let bio_result = update_bio(bio_text).await;
    handle_save_results(bio_result, img_result, toast, response_msg);
    is_saving.set(false);
});
```

---

### Task 9: E2E Tests (Playwright)

**Scope:** Profile-specific E2E tests using the mock backend.

**Test cases:**

| # | Test | Route | What it verifies |
|---|------|-------|------------------|
| 1 | My Profile loads with user data | `/profile` | Header shows username, slug, bio, stats; post list renders |
| 2 | Infinite scroll loads more posts | `/profile` | Scroll to bottom triggers second batch |
| 3 | View Profile loads another user | `/profile/:slug` | Displays target user's data, follow button visible |
| 4 | Follow/unfollow toggle | `/profile/:slug` | Button state changes, optimistic update |
| 5 | `/edit-profile` redirects to `/settings` | `/edit-profile` | URL changes to `/settings` |
| 6 | `?user=<uuid>` redirect | `/profile?user=<uuid>` | Redirects to `/u/<uuid>` |
| 7 | Relations modal pagination | `/profile` | Click followers count → modal → scroll loads more |
| 8 | Filter sidebar (content type) | `/profile` | Toggle "video" filter → only video posts shown |
| 9 | Block user | `/profile/:slug` | Block action → confirmation → UI updates |
| 10 | Profile not found | `/profile/nonexistent` | 404-style "Profile Not Found" displayed |

**Prerequisite:** Mock backend already supports `getProfile`, `listPosts(userid:)`, `toggleUserFollowStatus`, `blockUser`, `reportUser`.

---

## Code Quality Pass (Post-Implementation)

After Tasks 1–8 were implemented, a review identified and fixed the following issues:

### 1. Shared `use_infinite_scroll` Hook

**Problem:** The ~30-line IntersectionObserver + `callback.forget()` pattern was copy-pasted across 6 components.

**Solution:** Extracted `use_infinite_scroll(is_loading, has_more, load_fn) -> InfiniteScroll` into `src/hooks/use_infinite_scroll.rs`. The hook:
- Takes `RwSignal<bool>` for `is_loading`/`has_more` and an `impl Fn() + 'static` load callback
- Wraps the callback in `Rc` internally so callers don't need `Copy` closures
- Stores both the `IntersectionObserver` and the `Closure` in cleanup, fixing the memory leak from `callback.forget()`
- Returns `InfiniteScroll { loader_ref, is_loading, has_more }` for view-layer binding

**Applied to:** `PostList` (dashboard), `ProfilePostList`, `UserPostList`, `RelationsModal`, `AdList`, `TransactionHistory` — ~180 lines removed.

### 2. `use_navigate()` in Async Context (deactivate.rs)

**Problem:** `use_navigate()` was called inside a `spawn_local` async block in `DeactivateAccountPanel`. Leptos hooks must be called in the synchronous component body; calling them in async contexts is unsound and may panic in future Leptos versions.

**Fix:** Call `use_navigate()` at the point of use inside the event handler (synchronous context), not inside the async task.

### 3. Parallel Save Race Condition (settings/profile.rs)

**Problem:** The dual-`spawn_local` pattern with `RwSignal<Option<Result<...>>>` completion checks had a subtle race: the "no image" synchronous fast path could fire before the bio task resolved.

**Fix:** Single `spawn_local` with sequential awaits. No `futures` crate dependency needed.

---

## Implementation Order

```
Phase 1 — Quick Wins (Tasks 1, 4, 5, 8)           ~3 hours
  ├─ Task 1: /edit-profile redirect
  ├─ Task 4: ?user= query param redirect
  ├─ Task 5: ProfileWidget → auth context
  └─ Task 8: Parallel bio/image save

Phase 2 — Core UX (Tasks 2, 3)                     ~4 hours
  ├─ Task 2: Infinite scroll on both profile pages
  └─ Task 3: Filter sidebar wired to profile pages

Phase 3 — Polish (Tasks 6, 7)                       ~4 hours
  ├─ Task 6: Relations modal infinite scroll
  └─ Task 7: Deactivate account UI

Phase 4 — Testing (Task 9)                          ~4 hours
  └─ Task 9: E2E Playwright tests
```

---

## Convergence Updates (on completion)

After this sprint, update `feature-convergence.md`:

| Feature | Before | After |
|---------|--------|-------|
| My Profile | 🟡 Mostly Implemented | ✅ Implemented |
| View Profile | 🟡 Mostly Implemented | ✅ Implemented |
| Edit Profile | ❌ Not Started | ✅ Implemented (redirect to Settings) |
| Settings | ✅ Implemented (with gaps) | ✅ Implemented |
| Dashboard | 🟡 Mostly Implemented | 🟡 (profile widget gap closed; post overlay still TODO) |

**New convergence: ~79%** (8 ✅ / 20 features, up from 5 ✅)

---

## Dependencies

- **Mock backend:** All required queries/mutations already implemented (Phases 1–4 complete for users, posts, follow, block, report)
- **Filter components:** Fully built for dashboard, reusable as-is
- **IntersectionObserver pattern:** Proven in PostList, TransactionHistory, AdList — copy-adapt
- **Auth context:** Exists and provides user ID; may need a `profile` field or lightweight cache for ProfileWidget
