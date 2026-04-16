# Version History Implementation Plan

**Feature:** Version History  
**Priority:** #15 (quick win — simple static page)  
**Status:** ✅ Implemented  
**Created:** 2026-04-16  
**Completed:** 2026-04-16  
**Plan Quality:** ⭐⭐⭐⭐ (4/5)

---

## Overview

Implement the Version History page for the Leptos frontend. This page displays release notes and changelog information for the Peer Network web application, loaded from a static JSON file. Users can browse through different versions and view detailed release notes for each.

### Goals

1. Full parity with legacy `version_history.php` + `js/version_history.js`
2. Load version data from `/json/version_releases.json` (static asset)
3. Two-panel layout: version list (left) + version details (right)
4. Interactive version selection with active state
5. Responsive layout for mobile/desktop
6. Auth-guarded page (requires authentication)

---

## Scope

### In Scope

- [x] Version History page (`/version-history` route)
- [x] Auth guard (redirect to `/login` if unauthenticated)
- [x] Fetch and parse `/json/version_releases.json` on mount
- [x] Left panel: version list with clickable items
- [x] Right panel: version details (header, changes, links)
- [x] Default selection of latest version on load
- [x] Active state styling for selected version
- [x] External links to Wiki/documentation
- [x] Loading state while fetching JSON
- [x] Error state if fetch fails
- [x] Back button to return to Settings/Profile
- [x] Right sidebar with standard widgets (Profile, Menu, Version)
- [x] Mobile footer navigation

### Out of Scope (Future Work)

- CMS/admin interface for editing releases
- Search/filter functionality for versions
- Version comparison view
- Push notifications for new versions
- Deep linking to specific versions (`/version-history/v12`)

---

## Legacy Implementation Analysis

### Files

| File | Lines | Purpose |
|------|-------|---------|
| `version_history.php` | 66 | Page template with settings-like layout |
| `js/version_history.js` | 59 | Fetch JSON, render version list + details |
| `json/version_releases.json` | ~500 | Version release data (12+ versions) |
| `css/settings.css` | shared | Reused for settings-like layout |

### Data Structure (`version_releases.json`)

```json
{
  "id": "v12",
  "version": "Version 1.12.0",
  "date": "06 Jan 2026",
  "changes": [
    {
      "title": "Feature Title",
      "description": ["Bullet point 1", "Bullet point 2"]
    }
  ],
  "links": [
    { "label": "Web Wiki", "href": "https://..." },
    { "label": "Backend Wiki", "href": "https://..." }
  ]
}
```

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│ HEADER: [Logo] Settings                                    │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│              │  ┌───────┬─────────────┐│                   │
│ [Back to     │  │ v1.12 │ Version 1.12││  - Profile widget │
│  Settings]   │  │ v1.11 │ 06 Jan 2026 ││  - Main menu      │
│              │  │ v1.10 │             ││  - New post btn   │
│              │  │ v1.9  │ ─────────── ││  - Version        │
│              │  │ v1.8  │ Feature 1   ││                   │
│              │  │ ...   │ • bullet 1  ││                   │
│              │  │       │ • bullet 2  ││                   │
│              │  │       │             ││                   │
│              │  │       │ ─────────── ││                   │
│              │  │       │ [Wiki] [API]││                   │
│              │  └───────┴─────────────┘│                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### Version Detail Panel

```
┌─────────────────────────────────────────────────────────────┐
│ Version 1.12.0                                              │
│ 06 Jan 2026                                                 │
├─────────────────────────────────────────────────────────────┤
│ A clear Transaction History                                 │
│ Your wallet now gives you a complete, transparent view...   │
├─────────────────────────────────────────────────────────────┤
│ Each transaction includes:                                  │
│ • Total Peer Token amount (including fees)                  │
│ • Fee breakdown (Peer Bank, Burns, Inviter)                 │
│ • Date and time stamp                                       │
├─────────────────────────────────────────────────────────────┤
│ [Web Wiki]  [Backend Wiki]                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 1: Data Types & Fetching

| # | Task | Notes |
|---|------|-------|
| 1.1 | Create `src/models/version.rs` with version types | `VersionRelease`, `VersionChange`, `VersionLink` structs |
| 1.2 | Implement JSON fetch function | Client-side fetch from `/json/version_releases.json` |
| 1.3 | Add error handling for fetch failures | Network errors, parse errors |

### Phase 2: Page Component

| # | Task | Notes |
|---|------|-------|
| 2.1 | Create `src/pages/version_history.rs` page | Auth-guarded, settings-like layout |
| 2.2 | Add `/version-history` route to router | In `src/app.rs` |
| 2.3 | Implement page header | Logo + "Settings" title (matches legacy) |
| 2.4 | Add left sidebar with back button | "Back to Settings" link |
| 2.5 | Add right sidebar with standard widgets | Reuse `ProfileWidget`, `MainMenu`, `VersionWidget` |
| 2.6 | Add mobile footer | Reuse `MobileFooter` pattern |

### Phase 3: Version List Component

| # | Task | Notes |
|---|------|-------|
| 3.1 | Create `src/components/version_history/mod.rs` | Module structure |
| 3.2 | Create `src/components/version_history/version_list.rs` | List of version buttons |
| 3.3 | Implement version button with active state | Clickable, highlighted when selected |
| 3.4 | Wire selected version signal | `RwSignal<Option<usize>>` for selected index |
| 3.5 | Default select first version on load | Latest version shown by default |

### Phase 4: Version Detail Component

| # | Task | Notes |
|---|------|-------|
| 4.1 | Create `src/components/version_history/version_detail.rs` | Detail view component |
| 4.2 | Implement header section | Version number + date |
| 4.3 | Implement changes section | Loop through `changes[]`, each with title + description bullets |
| 4.4 | Implement links section | External link buttons |
| 4.5 | Handle empty state | "Select a version" placeholder |

### Phase 5: Styling

| # | Task | Notes |
|---|------|-------|
| 5.1 | Create `src/style/pages/_version_history.scss` | Page-specific styles |
| 5.2 | Import in `src/style/main.scss` | Add to SCSS imports |
| 5.3 | Style version list buttons | Active state, hover effects |
| 5.4 | Style version detail panel | Header, content sections, link buttons |
| 5.5 | Responsive breakpoints | Mobile stacked layout |

### Phase 6: Integration & Polish

| # | Task | Notes |
|---|------|-------|
| 6.1 | Add link to Version History from Settings | In footer or version widget |
| 6.2 | Loading skeleton while fetching | Skeleton list + detail placeholders |
| 6.3 | Error state display | "Failed to load version history" message |
| 6.4 | Test with mock data | Verify all version formats render correctly |

---

## Technical Details

### Data Types (`src/models/version.rs`)

```rust
use serde::{Deserialize, Serialize};

/// A single changelog entry within a version release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub title: String,
    pub description: Vec<String>,
}

/// An external link associated with a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionLink {
    pub label: String,
    pub href: String,
}

/// A version release containing changelog information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRelease {
    pub id: String,
    pub version: String,
    pub date: String,
    pub changes: Vec<VersionChange>,
    pub links: Vec<VersionLink>,
}
```

### Fetch Function

```rust
use gloo_net::http::Request;

/// Fetch version releases from static JSON file.
pub async fn fetch_version_releases() -> Result<Vec<VersionRelease>, String> {
    let response = Request::get("/json/version_releases.json")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }
    
    response
        .json::<Vec<VersionRelease>>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}
```

### Page Component Structure

```rust
#[component]
pub fn VersionHistoryPage() -> impl IntoView {
    // Selected version index
    let selected_idx = RwSignal::new(Some(0usize));
    
    // Fetch version releases on mount
    let versions_resource = Resource::new(
        || (),
        |_| async move { fetch_version_releases().await }
    );
    
    view! {
        <Title text="Version History - Peer Network"/>
        <AuthGuard>
            <div id="version_history" class="site_layout version_history">
                <VersionHistoryHeader/>
                
                <aside class="left-sidebar left-sidebar-profile">
                    <div class="inner-scroll">
                        <BackButton href="/settings" label="Back to Settings"/>
                    </div>
                </aside>
                
                <main class="site-main site_main_versionHistory">
                    <div class="setting-layout">
                        <Suspense fallback=move || view! { <VersionListSkeleton/> }>
                            {move || versions_resource.get().map(|result| match result {
                                Ok(versions) => view! {
                                    <VersionList
                                        versions=versions.clone()
                                        selected_idx=selected_idx
                                    />
                                    <VersionDetail
                                        versions=versions
                                        selected_idx=selected_idx
                                    />
                                }.into_any(),
                                Err(e) => view! {
                                    <VersionHistoryError message=e/>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </main>
                
                <RightSidebar/>
                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}
```

### Version List Component

```rust
#[component]
pub fn VersionList(
    versions: Vec<VersionRelease>,
    selected_idx: RwSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <div class="setting-menu left_versionHistory">
            <For
                each=move || versions.clone().into_iter().enumerate()
                key=|(_, v)| v.id.clone()
                children=move |(idx, version)| {
                    let is_active = move || selected_idx.get() == Some(idx);
                    view! {
                        <a
                            class:active=is_active
                            on:click=move |_| selected_idx.set(Some(idx))
                        >
                            {version.version.clone()}
                        </a>
                    }
                }
            />
        </div>
    }
}
```

### Version Detail Component

```rust
#[component]
pub fn VersionDetail(
    versions: Vec<VersionRelease>,
    selected_idx: RwSignal<Option<usize>>,
) -> impl IntoView {
    let selected_version = move || {
        selected_idx.get().and_then(|idx| versions.get(idx).cloned())
    };
    
    view! {
        <div class="setting-content right_versionHistory">
            {move || match selected_version() {
                Some(release) => view! {
                    <div class="releaseVersion_header">
                        <h1 class="xxl_font_size bold">{release.version}</h1>
                        <span>{release.date}</span>
                    </div>
                    
                    <For
                        each=move || release.changes.clone()
                        key=|c| c.title.clone()
                        children=|change| view! {
                            <div class="releaseVersion_content">
                                <h2 class="md_font_size bold">{change.title}</h2>
                                <For
                                    each=move || change.description.clone()
                                    key=|d| d.clone()
                                    children=|desc| view! { <p>{desc}</p> }
                                />
                            </div>
                        }
                    />
                    
                    <div class="releaseVersion_btns">
                        <For
                            each=move || release.links.clone()
                            key=|l| l.href.clone()
                            children=|link| view! {
                                <a
                                    href=link.href
                                    target="_blank"
                                    class="button btn-transparent Versionbtn-link"
                                >
                                    {link.label}
                                </a>
                            }
                        />
                    </div>
                }.into_any(),
                None => view! {
                    <p class="empty-state">"Select a version to view details"</p>
                }.into_any(),
            }}
        </div>
    }
}
```

---

## File Changes

### New Files

| File | Purpose |
|------|---------|
| `src/models/version.rs` | Version data types |
| `src/pages/version_history.rs` | Page component |
| `src/components/version_history/mod.rs` | Component module |
| `src/components/version_history/version_list.rs` | Version list component |
| `src/components/version_history/version_detail.rs` | Version detail component |
| `src/style/pages/_version_history.scss` | Page styles |

### Modified Files

| File | Changes |
|------|---------|
| `src/models/mod.rs` | Add `pub mod version;` |
| `src/pages/mod.rs` | Add `pub mod version_history;` |
| `src/components/mod.rs` | Add `pub mod version_history;` |
| `src/app.rs` | Add `/version-history` route |
| `src/style/main.scss` | Import `_version_history.scss` |

---

## Testing Strategy

### Manual Testing

| # | Scenario | Expected Result |
|---|----------|-----------------|
| 1 | Navigate to `/version-history` unauthenticated | Redirect to `/login` |
| 2 | Navigate to `/version-history` authenticated | Page loads with version list |
| 3 | Page load | Latest version (v1.12) selected by default |
| 4 | Click different version | Details panel updates, active state changes |
| 5 | JSON fetch fails | Error message displayed |
| 6 | External link click | Opens in new tab |
| 7 | Mobile viewport | Responsive stacked layout |
| 8 | "Back to Settings" click | Navigates to `/settings` |

### Future E2E Tests

```rust
#[tokio::test]
async fn version_history_loads_versions() {
    // Navigate to /version-history
    // Assert version list contains expected versions
    // Assert first version is selected
}

#[tokio::test]
async fn version_history_switches_versions() {
    // Click on version v1.11
    // Assert details panel shows v1.11 content
    // Assert v1.11 button has active class
}

#[tokio::test]
async fn version_history_requires_auth() {
    // Navigate to /version-history without auth
    // Assert redirect to /login
}
```

---

## Dependencies

- **gloo-net**: For client-side HTTP fetch (already in project)
- **serde/serde_json**: For JSON deserialization (already in project)
- No new GraphQL operations needed
- No mock backend changes needed

---

## Definition of Done

- [x] All 6 phases implemented
- [x] Page renders correctly with all 12+ versions
- [x] Version selection works (click updates details)
- [x] Mobile responsive layout
- [x] Loading and error states
- [ ] Accessible (keyboard navigation)
- [x] No clippy warnings
- [ ] Manual testing scenarios pass
- [x] Feature convergence tracker updated

---

## Estimated Effort

| Phase | Estimate |
|-------|----------|
| Phase 1: Data Types | 30 min |
| Phase 2: Page | 45 min |
| Phase 3: Version List | 30 min |
| Phase 4: Version Detail | 45 min |
| Phase 5: Styling | 45 min |
| Phase 6: Integration | 30 min |
| **Total** | **~4 hours** |

---

## Notes

- This is a simple "quick win" feature with no backend dependencies
- Layout reuses patterns from Settings page
- Static JSON means no E2E mock backend changes required
- Could later be extended with deep linking (`/version-history/v12`)
- Consider caching the JSON fetch for performance
