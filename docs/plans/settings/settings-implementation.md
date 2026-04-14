# Settings Implementation Plan

**Feature:** Settings  
**Priority:** #9 (after Wallet)  
**Status:** 📋 Planning  
**Created:** 2026-04-14

---

## Overview

Implement the user settings page for the Leptos frontend. This is a multi-section page allowing authenticated users to manage their profile, account credentials, content preferences, and account lifecycle (logout, deactivation). The page uses a tabbed navigation pattern with distinct sections.

### Goals

1. Full parity with legacy `profileSettings.php` user experience
2. Four settings tabs: Profile, Notifications, Preferences, Content
3. Profile editing: avatar upload with zoom/crop preview, biography, username change
4. Account management: change password, change email
5. Content filtering: toggle reported content visibility
6. Account actions: logout with confirmation, deactivate profile
7. Responsive layout with sidebar navigation
8. Client-side validation with server-side response handling
9. Auth-guarded page (requires authentication)

---

## Scope

### In Scope

- [ ] Settings page (`/settings` route)
- [ ] Auth guard (redirect to `/login` if unauthenticated)
- [ ] Tab navigation (Profile, Notifications, Preferences, Content)
- [ ] **Profile Settings tab:**
  - [ ] Display current avatar with change button
  - [ ] Image upload with preview modal (zoom slider)
  - [ ] Biography textarea (max 5000 chars)
  - [ ] Display current username with change link
  - [ ] Save profile changes (avatar + bio in parallel)
  - [ ] Change username sub-panel (username + password confirmation)
  - [ ] Change password sub-panel (old password, new password with strength, confirm)
  - [ ] Change email sub-panel (new email + password confirmation)
- [ ] **Content Settings tab:**
  - [ ] Reported content toggle (`MYGRANDMALIKES` / `MYGRANDMAHATES`)
  - [ ] Confirmation dialog before toggling
  - [ ] Success modal after update
- [ ] **Notification Settings tab:**
  - [ ] Placeholder (stub — not yet implemented in legacy)
- [ ] **Preferences tab:**
  - [ ] Placeholder (stub — not yet implemented in legacy)
- [ ] **Account Actions:**
  - [ ] Logout with confirmation modal
  - [ ] Deactivate profile (delete account)
- [ ] Loading and success/error states for all forms
- [ ] Password strength indicator (reuse existing component)
- [ ] Confirm password validation (reuse existing component)

### Out of Scope (Future Work)

- Notification preferences (backend not implemented)
- Theme/language preferences (backend not implemented)
- Two-factor authentication
- Connected accounts / social login
- Data export / GDPR tools

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `profileSettings.php` | Main settings page layout |
| `template-parts/content-parts/settings-menu.php` | Tab navigation menu |
| `template-parts/settings/editProfile.php` | Profile editing forms |
| `template-parts/settings/notification.php` | Notification stub |
| `template-parts/settings/content.php` | Content filtering toggle |
| `template-parts/settings/preferences.php` | Preferences stub |
| `js/settings/index.js` | Tab switching logic |
| `js/settings/editProfile.js` | Profile forms, submissions, logout |
| `js/settings/content.js` | Reported content toggle logic |
| `js/settings/preferences.js` | Empty (not implemented) |
| `js/settings/notification.js` | Empty (not implemented) |
| `js/password.js` | Password strength indicator |
| `js/confirmPassword.js` | Confirm password matching |
| `css/settings.css` | Settings layout and tab styles |
| `css/edit-profile.css` | Profile form styles |
| `css/password.css` | Password strength bar styles |

### Layout Structure

```
┌──────────────────────────────────────────────────────────────┐
│ Header: Logo + "Settings"                                    │
├────────┬─────────────────────────────────────────────────────┤
│ Left   │  Main Content                                       │
│ Side   │  ┌──────────────┬──────────────────────────────────┐│
│ bar    │  │ Tab Menu     │  Active Tab Content              ││
│        │  │ (30%)        │  (65%)                           ││
│ [Back] │  │              │                                  ││
│        │  │ ● Profile *  │  ┌──────────────────────────┐   ││
│        │  │ ○ Notif.     │  │ Profile Picture           │   ││
│        │  │ ○ Prefs.     │  │ [Change Picture]          │   ││
│        │  │ ○ Content    │  │                            │   ││
│        │  │              │  │ Description                │   ││
│        │  │ [Log Out]    │  │ [textarea]                 │   ││
│        │  │ [Deactivate] │  │                            │   ││
│        │  │              │  │ Username: @name [Change]   │   ││
│        │  │              │  │                            │   ││
│        │  │              │  │ [Save Changes]             │   ││
│        │  │              │  │                            │   ││
│        │  │              │  │ [Change Password]          │   ││
│        │  │              │  │ [Change E-mail]            │   ││
│        │  │              │  └──────────────────────────┘   ││
│        │  └──────────────┴──────────────────────────────────┘│
├────────┴──────────────────────────────────────┬──────────────┤
│                                                │ Right       │
│                                                │ Sidebar     │
│                                                │ (widgets)   │
└────────────────────────────────────────────────┴──────────────┘
```

### Sub-Panel Navigation (within Profile Settings)

The Profile Settings tab contains expandable sub-panels:

```
Default View:
  [Avatar] [Change Picture]
  [Biography textarea]
  Username: @name [Change]
  [Save Changes]
  [Change Password] [Change E-mail]

→ Click "Change" → sub-panel slides in:
  ┌─────────────────────────┐
  │ Change username          │
  │ [New username input]     │
  │ [Password input]         │
  │ [Submit]                 │
  └─────────────────────────┘

→ Click "Change password" → sub-panel slides in:
  ┌─────────────────────────┐
  │ Change password          │
  │ [Old password]           │
  │ [New password + strength]│
  │ [Confirm password]       │
  │ [Submit]                 │
  └─────────────────────────┘

→ Click "Change e-mail" → sub-panel slides in:
  ┌─────────────────────────┐
  │ Change e-mail            │
  │ [New email input]        │
  │ [Password input]         │
  │ [Submit]                 │
  └─────────────────────────┘
```

### Key Features

1. **Profile Picture Upload**
   - Click "Change Picture" opens file picker (images only)
   - Selected image shown in preview modal with zoom slider
   - Apply sets the image locally; saved on "Save Changes"
   - Image sent as base64 `data:image/*` to `updateProfileImage`

2. **Biography**
   - Textarea, max 5000 chars
   - Loaded from remote text file URL (fetched via `tempMedia()`)
   - Saved as base64 `data:text/plain;base64,...` to `updateBio`

3. **Username Change**
   - Requires new username + current password
   - Validation: 3–23 chars, alphanumeric/underscores/hyphens
   - Page reloads on success

4. **Password Change**
   - Requires old password, new password, confirm password
   - Client validation: min 8 chars, uppercase, digit, special char
   - Password strength bar indicator
   - Passwords must match

5. **Email Change**
   - Requires new email + current password
   - Client validation: email regex

6. **Content Filtering Toggle**
   - Toggle between `MYGRANDMALIKES` (strict) and `MYGRANDMAHATES` (lenient)
   - Confirmation dialog before switching
   - Success modal after update

7. **Logout**
   - Confirmation modal: "Are you sure you want to log out?"
   - Clears localStorage, sessionStorage, cookies
   - Redirects to login

8. **Account Deactivation**
   - Calls `deleteAccount` mutation with password confirmation
   - Soft-deletes account (status = 6)

---

## Backend API Reference

### `getProfile` Query (Load Current User Data)

```graphql
query GetProfile {
  getProfile {
    meta { status, ResponseCode }
    affectedRows {
      id
      username
      slug
      img
      biography
      visibilityStatus
    }
  }
}
```

| Code | Description |
|------|-------------|
| `11008` | Profile loaded successfully |
| `60501` | Not authenticated |

### `getUserInfo` Query (Load Preferences)

```graphql
query GetUserInfo {
  getUserInfo {
    meta { status, ResponseCode }
    affectedRows {
      userid
      invited
      userPreferences {
        contentFilteringSeverityLevel
      }
    }
  }
}
```

| Code | Description |
|------|-------------|
| `11009` | User data prepared successfully |
| `60501` | Not authenticated |

### `updateProfileImage` Mutation

```graphql
mutation UpdateProfileImage($img: String!) {
  updateProfileImage(img: $img) {
    status
    ResponseCode
  }
}
```

| Code | Description |
|------|-------------|
| `11004` | Profile picture updated |
| `30101` | Missing fields |
| `60501` | Not authenticated |

### `updateBio` Mutation

```graphql
mutation UpdateBio($biography: String!) {
  updateBio(biography: $biography) {
    status
    ResponseCode
  }
}
```

| Code | Description |
|------|-------------|
| `11003` | Bio updated |
| `30101` | Missing fields |
| `60501` | Not authenticated |

### `updateUsername` Mutation

```graphql
mutation UpdateUsername($username: String!, $password: String!) {
  updateUsername(username: $username, password: $password) {
    status
    ResponseCode
  }
}
```

| Code | Description |
|------|-------------|
| `11007` | Username changed |
| `30202` | Invalid username format |
| `30101` | Missing fields |
| `60501` | Not authenticated |

### `updatePassword` Mutation

```graphql
mutation UpdatePassword($password: String!, $expassword: String!) {
  updatePassword(password: $password, expassword: $expassword) {
    status
    ResponseCode
  }
}
```

| Code | Description |
|------|-------------|
| `11005` | Password updated |
| `30101` | Missing fields |
| `31001` | Old password does not match |
| `60501` | Not authenticated |

### `updateEmail` Mutation

```graphql
mutation UpdateEmail($email: String!, $password: String!) {
  updateEmail(email: $email, password: $password) {
    status
    ResponseCode
  }
}
```

| Code | Description |
|------|-------------|
| `11006` | Email updated |
| `30103` | Invalid email format |
| `60501` | Not authenticated |

### `updateUserPreferences` Mutation

```graphql
mutation UpdateUserPreferences($userPreferences: UserPreferencesInput) {
  updateUserPreferences(userPreferences: $userPreferences) {
    status
    ResponseCode
    affectedRows {
      contentFilteringSeverityLevel
    }
  }
}
```

Input:
```graphql
input UserPreferencesInput {
  contentFilteringSeverityLevel: ContentFilterType  # MYGRANDMALIKES | MYGRANDMAHATES
  shownOnboardings: [OnboardingType!]
}
```

| Code | Description |
|------|-------------|
| `11014` | Preferences updated |
| `60501` | Not authenticated |

### `deleteAccount` Mutation

```graphql
mutation DeleteAccount($password: String!) {
  deleteAccount(password: $password) {
    status
    ResponseCode
  }
}
```

| Code | Description |
|------|-------------|
| `11012` | Account deleted successfully |
| `30101` | Missing password |
| `31001` | Password does not match |
| `60501` | Not authenticated |

---

## Implementation Plan

### Phase 1: API Layer & Models

#### 1.1 Settings Models (`src/models/settings.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Response from profile update mutations (updateBio, updateProfileImage, etc.).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateResponse {
    pub status: String,
    pub response_code: String,
}

impl UpdateResponse {
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}

/// Response wrapping for updateUserPreferences.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesUpdateResponse {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    pub affected_rows: Option<UserPreferencesPayload>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesPayload {
    pub content_filtering_severity_level: String,
}

/// Input for content filtering preference.
#[derive(Debug, Clone, Serialize)]
pub enum ContentFilterLevel {
    /// Stricter: hides all flagged content.
    #[serde(rename = "MYGRANDMALIKES")]
    Strict,
    /// Lenient: shows placeholders for flagged content.
    #[serde(rename = "MYGRANDMAHATES")]
    Lenient,
}
```

#### 1.2 Settings API (`src/api/settings.rs`)

```rust
use leptos::prelude::*;

/// Update the user's profile image (base64-encoded).
#[server(UpdateProfileImage, "/api")]
pub async fn update_profile_image(img: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, UpdateProfileImageData, UPDATE_PROFILE_IMAGE_MUTATION};

    let token = get_access_token_from_cookies().await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "img": img });
    let data: UpdateProfileImageData = mutate(UPDATE_PROFILE_IMAGE_MUTATION, vars, Some(&token)).await?;

    if data.update_profile_image.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!("Error: {}", data.update_profile_image.response_code)))
    }
}

/// Update the user's biography.
#[server(UpdateBio, "/api")]
pub async fn update_bio(biography: String) -> Result<(), ServerFnError> {
    // Encode to base64 data URI format as expected by backend
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(biography.as_bytes());
    let data_uri = format!("data:text/plain;base64,{}", encoded);

    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, UpdateBioData, UPDATE_BIO_MUTATION};

    let token = get_access_token_from_cookies().await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "biography": data_uri });
    let data: UpdateBioData = mutate(UPDATE_BIO_MUTATION, vars, Some(&token)).await?;

    if data.update_bio.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!("Error: {}", data.update_bio.response_code)))
    }
}

/// Update the user's username. Requires current password.
#[server(UpdateUsername, "/api")]
pub async fn update_username(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, UpdateUsernameData, UPDATE_USERNAME_MUTATION};
    use crate::api::validation;

    // Server-side validation
    validation::validate_username(&username)?;

    let token = get_access_token_from_cookies().await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "username": username, "password": password });
    let data: UpdateUsernameData = mutate(UPDATE_USERNAME_MUTATION, vars, Some(&token)).await?;

    if data.update_username.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!("Error: {}", data.update_username.response_code)))
    }
}

/// Update the user's password. Requires current password.
#[server(UpdatePassword, "/api")]
pub async fn update_password(new_password: String, old_password: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, UpdatePasswordData, UPDATE_PASSWORD_MUTATION};
    use crate::api::validation;

    // Server-side validation
    validation::validate_password(&new_password)?;

    let token = get_access_token_from_cookies().await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "password": new_password, "expassword": old_password });
    let data: UpdatePasswordData = mutate(UPDATE_PASSWORD_MUTATION, vars, Some(&token)).await?;

    if data.update_password.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!("Error: {}", data.update_password.response_code)))
    }
}

/// Update the user's email. Requires current password.
#[server(UpdateEmail, "/api")]
pub async fn update_email(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, UpdateEmailData, UPDATE_EMAIL_MUTATION};
    use crate::api::validation;

    // Server-side validation
    validation::validate_email(&email)?;

    let token = get_access_token_from_cookies().await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "email": email, "password": password });
    let data: UpdateEmailData = mutate(UPDATE_EMAIL_MUTATION, vars, Some(&token)).await?;

    if data.update_email.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!("Error: {}", data.update_email.response_code)))
    }
}

/// Update user content filtering preferences.
#[server(UpdateContentPreferences, "/api")]
pub async fn update_content_preferences(severity_level: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, UpdatePreferencesData, UPDATE_PREFERENCES_MUTATION};

    let token = get_access_token_from_cookies().await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({
        "userPreferences": {
            "contentFilteringSeverityLevel": severity_level
        }
    });
    let data: UpdatePreferencesData = mutate(UPDATE_PREFERENCES_MUTATION, vars, Some(&token)).await?;

    if data.update_user_preferences.response_code == "11014" {
        Ok(())
    } else {
        Err(ServerFnError::new(format!("Error: {}", data.update_user_preferences.response_code)))
    }
}

/// Deactivate (soft-delete) the user's account. Requires password.
#[server(DeleteAccount, "/api")]
pub async fn delete_account(password: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, DeleteAccountData, DELETE_ACCOUNT_MUTATION};

    let token = get_access_token_from_cookies().await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "password": password });
    let data: DeleteAccountData = mutate(DELETE_ACCOUNT_MUTATION, vars, Some(&token)).await?;

    if data.delete_account.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!("Error: {}", data.delete_account.response_code)))
    }
}
```

#### 1.3 GraphQL Queries (`src/api/graphql.rs` additions)

```rust
pub const UPDATE_PROFILE_IMAGE_MUTATION: &str = r#"
    mutation UpdateProfileImage($img: String!) {
        updateProfileImage(img: $img) {
            status
            ResponseCode
        }
    }
"#;

pub const UPDATE_BIO_MUTATION: &str = r#"
    mutation UpdateBio($biography: String!) {
        updateBio(biography: $biography) {
            status
            ResponseCode
        }
    }
"#;

pub const UPDATE_USERNAME_MUTATION: &str = r#"
    mutation UpdateUsername($username: String!, $password: String!) {
        updateUsername(username: $username, password: $password) {
            status
            ResponseCode
        }
    }
"#;

pub const UPDATE_PASSWORD_MUTATION: &str = r#"
    mutation UpdatePassword($password: String!, $expassword: String!) {
        updatePassword(password: $password, expassword: $expassword) {
            status
            ResponseCode
        }
    }
"#;

pub const UPDATE_EMAIL_MUTATION: &str = r#"
    mutation UpdateEmail($email: String!, $password: String!) {
        updateEmail(email: $email, password: $password) {
            status
            ResponseCode
        }
    }
"#;

pub const UPDATE_PREFERENCES_MUTATION: &str = r#"
    mutation UpdateUserPreferences($userPreferences: UserPreferencesInput) {
        updateUserPreferences(userPreferences: $userPreferences) {
            status
            ResponseCode
            affectedRows {
                contentFilteringSeverityLevel
            }
        }
    }
"#;

pub const DELETE_ACCOUNT_MUTATION: &str = r#"
    mutation DeleteAccount($password: String!) {
        deleteAccount(password: $password) {
            status
            ResponseCode
        }
    }
"#;

// Deserialization wrappers
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileImageData {
    pub update_profile_image: UpdateResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBioData {
    pub update_bio: UpdateResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsernameData {
    pub update_username: UpdateResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasswordData {
    pub update_password: UpdateResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmailData {
    pub update_email: UpdateResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreferencesData {
    pub update_user_preferences: UserPreferencesUpdateResponse,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAccountData {
    pub delete_account: UpdateResponse,
}
```

### Phase 2: Settings Page & Tab Navigation

#### 2.1 Route Setup (`src/app.rs`)

```rust
// Add route
<Route path=StaticSegment("settings") view=SettingsPage/>
```

#### 2.2 Settings Page (`src/pages/settings.rs`)

```rust
use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::auth_guard::AuthGuard;
use crate::components::settings::{
    SettingsMenu, ProfileSettings, NotificationSettings,
    ContentSettings, PreferencesSettings,
};
use crate::api::profile::get_profile;

/// Active settings tab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsTab {
    Profile,
    Notifications,
    Preferences,
    Content,
}

/// Settings page — auth-guarded, tabbed layout.
#[component]
pub fn SettingsPage() -> impl IntoView {
    let active_tab = RwSignal::new(SettingsTab::Profile);

    // Load current user profile for the profile settings tab
    let profile_resource = Resource::new(
        || (),
        |_| async move { get_profile(None, None).await }
    );

    view! {
        <AuthGuard>
            <Title text="Settings - Peer Network"/>
            <div id="edit-profile" class="site_layout">
                <header class="site-header header-profile">
                    <img class="logo" src="/svg/logo_sw.svg" alt="Peer Network"/>
                    <h1>"Settings"</h1>
                </header>

                <aside class="left-sidebar left-sidebar-profile">
                    <div class="inner-scroll">
                        <div class="profile-back-button">
                            <a href="/profile" class="button btn-transparent">"Back to Profile"</a>
                        </div>
                    </div>
                </aside>

                <main class="site-main site-main-edit-profile">
                    <div class="setting-layout">
                        <SettingsMenu
                            active_tab=active_tab
                            on_tab_change=move |tab| active_tab.set(tab)
                        />

                        <Suspense fallback=move || view! { <SettingsSkeleton/> }>
                            {move || {
                                let tab = active_tab.get();
                                match tab {
                                    SettingsTab::Profile => view! {
                                        <ProfileSettings profile_resource=profile_resource/>
                                    }.into_any(),
                                    SettingsTab::Notifications => view! {
                                        <NotificationSettings/>
                                    }.into_any(),
                                    SettingsTab::Preferences => view! {
                                        <PreferencesSettings/>
                                    }.into_any(),
                                    SettingsTab::Content => view! {
                                        <ContentSettings/>
                                    }.into_any(),
                                }
                            }}
                        </Suspense>
                    </div>
                </main>

                <aside class="right-sidebar right-sidebar-edit-profile">
                    <div class="inner-scroll">
                        // Sidebar widgets (profile, menu, etc.)
                    </div>
                </aside>
            </div>
        </AuthGuard>
    }
}

#[component]
fn SettingsSkeleton() -> impl IntoView {
    view! {
        <div class="setting-content">
            <div class="skeleton-block" style="height: 200px; border-radius: 50%;"/>
            <div class="skeleton-block" style="height: 120px;"/>
            <div class="skeleton-block" style="height: 60px;"/>
        </div>
    }
}
```

#### 2.3 Settings Menu (`src/components/settings/menu.rs`)

```rust
use leptos::prelude::*;

use crate::pages::settings::SettingsTab;
use crate::state::auth::use_auth_context;

/// Settings tab navigation menu.
#[component]
pub fn SettingsMenu(
    active_tab: RwSignal<SettingsTab>,
    on_tab_change: impl Fn(SettingsTab) + 'static + Copy,
) -> impl IntoView {
    let auth = use_auth_context();
    let show_logout_modal = RwSignal::new(false);

    let tabs = vec![
        (SettingsTab::Profile, "Profile Settings"),
        (SettingsTab::Notifications, "Notification Settings"),
        (SettingsTab::Preferences, "Preferences"),
        (SettingsTab::Content, "Content Settings"),
    ];

    view! {
        <div class="setting-menu">
            <ul>
                {tabs.into_iter().map(|(tab, label)| {
                    view! {
                        <li
                            class:active=move || active_tab.get() == tab
                            on:click=move |_| on_tab_change(tab)
                        >
                            <a href="#" class="md_font_size"
                                on:click=|e| e.prevent_default()
                            >
                                {label}
                            </a>
                        </li>
                    }
                }).collect_view()}

                // Logout button
                <li class="not-menu-item">
                    <a href="#" class="md_font_size red-btn"
                        on:click=move |e| {
                            e.prevent_default();
                            show_logout_modal.set(true);
                        }
                    >
                        "Log Out"
                    </a>
                </li>

                // Deactivate button
                <li class="not-menu-item">
                    <a href="/deactivate" class="md_font_size red-btn"
                        on:click=|e| e.prevent_default()
                    >
                        "Deactivate profile"
                    </a>
                </li>
            </ul>

            // Logout confirmation modal
            <Show when=move || show_logout_modal.get()>
                <LogoutModal
                    on_cancel=move || show_logout_modal.set(false)
                    on_confirm=move || {
                        auth.logout_action.dispatch(());
                    }
                />
            </Show>
        </div>
    }
}

/// Logout confirmation modal.
#[component]
fn LogoutModal(
    on_cancel: impl Fn() + 'static,
    on_confirm: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="modal-overlay">
            <div class="logOut-pop">
                <img src="/svg/Union.svg" alt="logout"/>
                <p class="xl_font_size bold">"Are you sure you want to log out?"</p>
                <div class="button-row">
                    <button class="btn-white" on:click=move |_| on_cancel()>"Cancel"</button>
                    <button class="btn-red-transparent" on:click=move |_| on_confirm()>"Log Out"</button>
                </div>
            </div>
        </div>
    }
}
```

### Phase 3: Profile Settings Components

#### 3.1 Profile Settings (`src/components/settings/profile.rs`)

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::settings::{update_bio, update_profile_image};
use crate::components::toast::use_toast;
use crate::models::profile::Profile;

/// Active sub-panel within profile settings.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ProfileSubPanel {
    Main,
    ChangeUsername,
    ChangePassword,
    ChangeEmail,
}

/// Profile Settings tab content.
#[component]
pub fn ProfileSettings(
    profile_resource: Resource<Result<Profile, ServerFnError>>,
) -> impl IntoView {
    let active_panel = RwSignal::new(ProfileSubPanel::Main);

    view! {
        <div id="profile-settings" class="setting-content">
            {move || profile_resource.get().map(|result| match result {
                Ok(profile) => view! {
                    <Show when=move || active_panel.get() == ProfileSubPanel::Main>
                        <MainProfilePanel
                            profile=profile.clone()
                            on_change_username=move || active_panel.set(ProfileSubPanel::ChangeUsername)
                            on_change_password=move || active_panel.set(ProfileSubPanel::ChangePassword)
                            on_change_email=move || active_panel.set(ProfileSubPanel::ChangeEmail)
                        />
                    </Show>
                    <Show when=move || active_panel.get() == ProfileSubPanel::ChangeUsername>
                        <ChangeUsernamePanel on_back=move || active_panel.set(ProfileSubPanel::Main)/>
                    </Show>
                    <Show when=move || active_panel.get() == ProfileSubPanel::ChangePassword>
                        <ChangePasswordPanel on_back=move || active_panel.set(ProfileSubPanel::Main)/>
                    </Show>
                    <Show when=move || active_panel.get() == ProfileSubPanel::ChangeEmail>
                        <ChangeEmailPanel on_back=move || active_panel.set(ProfileSubPanel::Main)/>
                    </Show>
                }.into_any(),
                Err(_) => view! {
                    <p class="error">"Failed to load profile."</p>
                }.into_any(),
            })}
        </div>
    }
}

/// Main profile panel: avatar, bio, username display, save button.
#[component]
fn MainProfilePanel(
    profile: Profile,
    on_change_username: impl Fn() + 'static,
    on_change_password: impl Fn() + 'static,
    on_change_email: impl Fn() + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let biography = RwSignal::new(String::new());
    let image_data = RwSignal::new(Option::<String>::None);
    let avatar_src = RwSignal::new(
        profile.img.clone().unwrap_or_else(|| "/svg/noname.svg".to_string())
    );
    let is_saving = RwSignal::new(false);
    let show_image_modal = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    // Load biography text from remote URL
    let bio_path = profile.biography.clone();
    // Biography is loaded asynchronously from its text file URL

    let on_save = move |_| {
        if is_saving.get() { return; }
        is_saving.set(true);

        let bio_text = biography.get();
        let img = image_data.get();

        spawn_local(async move {
            // Run both updates in parallel
            let (img_result, bio_result) = futures::join!(
                async {
                    if let Some(img_data) = img {
                        update_profile_image(img_data).await
                    } else {
                        Ok(())
                    }
                },
                update_bio(bio_text),
            );

            match (&img_result, &bio_result) {
                (Ok(()), Ok(())) => {
                    toast.success("Profile updated successfully!");
                    response_msg.set(Some(("Profile saved.".to_string(), true)));
                }
                _ => {
                    toast.error("One or more updates failed.");
                    response_msg.set(Some(("Save failed.".to_string(), false)));
                }
            }
            is_saving.set(false);
        });
    };

    view! {
        <div class="profile-widget active">
            <div class="edit-profile">
                // Avatar section
                <div class="profile_picture">
                    <div class="cropContainer">
                        <img
                            class="profile-picture my-profile-picture"
                            src=move || avatar_src.get()
                            alt="Profile Picture"
                        />
                    </div>
                    <a href="#" class="button change-picture"
                        on:click=move |e| {
                            e.prevent_default();
                            show_image_modal.set(true);
                        }
                    >
                        "Change Picture"
                    </a>
                </div>

                // Biography
                <div class="profile-fields">
                    <div class="input-field transparent">
                        <label>"Description"</label>
                        <textarea
                            cols="40" rows="5" maxlength="5000"
                            class="input-textarea"
                            placeholder="Write a description to your profile..."
                            prop:value=move || biography.get()
                            on:input=move |ev| biography.set(event_target_value(&ev))
                        />
                    </div>

                    // Response message
                    {move || response_msg.get().map(|(msg, success)| {
                        view! {
                            <div class="response_msg" class:success=success class:error=!success>
                                {msg}
                            </div>
                        }
                    })}

                    // Username display
                    <div class="input-field username-row transparent">
                        <label>"Username"</label>
                        <span>"@"<span>{profile.username.clone()}</span></span>
                        <a href="#" on:click=move |e| {
                            e.prevent_default();
                            on_change_username();
                        }>"Change"</a>
                    </div>
                </div>

                <button
                    class="full-width-btn btn-white"
                    prop:disabled=move || is_saving.get()
                    on:click=on_save
                >
                    {move || if is_saving.get() { "Saving..." } else { "Save Changes" }}
                </button>
            </div>

            // Credential change buttons
            <div class="profile-setting-buttons">
                <div class="change-password-email">
                    <a href="#" class="button change-pass-btn btn-transparent"
                        on:click=move |e| { e.prevent_default(); on_change_password(); }
                    >
                        "Change password"
                    </a>
                    <a href="#" class="button change-email-btn btn-transparent"
                        on:click=move |e| { e.prevent_default(); on_change_email(); }
                    >
                        "Change e-mail"
                    </a>
                </div>
            </div>

            // Image upload modal
            <Show when=move || show_image_modal.get()>
                <ImageUploadModal
                    on_apply=move |data_url: String| {
                        avatar_src.set(data_url.clone());
                        image_data.set(Some(data_url));
                        show_image_modal.set(false);
                    }
                    on_cancel=move || show_image_modal.set(false)
                />
            </Show>
        </div>
    }
}
```

#### 3.2 Image Upload Modal (`src/components/settings/image_modal.rs`)

```rust
use leptos::prelude::*;

/// Image preview modal with zoom slider.
#[component]
pub fn ImageUploadModal(
    on_apply: impl Fn(String) + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let preview_src = RwSignal::new(String::new());
    let zoom_level = RwSignal::new(1.0f64);

    let on_file_change = move |ev: web_sys::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let input = ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                        if let Ok(result) = reader_clone.result() {
                            if let Some(data_url) = result.as_string() {
                                preview_src.set(data_url);
                            }
                        }
                    }) as Box<dyn FnMut(_)>);
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    reader.read_as_data_url(&file).unwrap();
                }
            }
        }
    };

    view! {
        <div class="modal" style="display: flex;">
            <div class="modal-content">
                <span class="closeBtn" on:click=move |_| on_cancel()>
                    <img src="/svg/close.svg" alt="Close"/>
                </span>
                <h2 class="modal-content-heading">"Preview"</h2>

                // File input (hidden, triggered by button)
                <input
                    type="file"
                    accept="image/*"
                    style="display: none;"
                    id="settings-file-input"
                    on:change=on_file_change
                />

                <Show when=move || !preview_src.get().is_empty()>
                    <div class="image-preview-wrapper">
                        <img
                            src=move || preview_src.get()
                            alt="Preview"
                            style=move || format!("transform: scale({})", zoom_level.get())
                        />
                    </div>
                    <div class="img-zoom">
                        <label>"Zoom"</label>
                        <input
                            type="range" min="1" max="3" step="0.1"
                            prop:value=move || zoom_level.get().to_string()
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                    zoom_level.set(val);
                                }
                            }
                        />
                    </div>
                </Show>

                <div class="button-row">
                    <button class="btn-transparent" on:click=move |_| on_cancel()>"Cancel"</button>
                    <button
                        class="btn-blue"
                        prop:disabled=move || preview_src.get().is_empty()
                        on:click=move |_| on_apply(preview_src.get())
                    >
                        "Apply"
                    </button>
                </div>
            </div>
        </div>
    }
}
```

#### 3.3 Change Username Panel (`src/components/settings/change_username.rs`)

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::settings::update_username;
use crate::components::toast::use_toast;

#[component]
pub fn ChangeUsernamePanel(on_back: impl Fn() + 'static) -> impl IntoView {
    let toast = use_toast();
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    let can_submit = move || {
        !username.get().trim().is_empty()
            && !password.get().trim().is_empty()
            && !is_submitting.get()
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !can_submit() { return; }

        is_submitting.set(true);
        let new_username = username.get().trim().to_string();
        let pw = password.get().clone();

        spawn_local(async move {
            match update_username(new_username, pw).await {
                Ok(()) => {
                    toast.success("Username updated!");
                    username.set(String::new());
                    password.set(String::new());
                    response_msg.set(Some(("Username changed successfully.".to_string(), true)));
                    // Navigate or reload after short delay
                }
                Err(e) => {
                    response_msg.set(Some((e.to_string(), false)));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="profile-widget active">
            <div class="edit-profile change-username">
                <h2 class="section_heading">"Change username"</h2>
                <form class="form-container" on:submit=on_submit>
                    <div class="profile-fields">
                        <div class="input-field">
                            <input
                                type="text"
                                class="input-text"
                                placeholder="Enter new username"
                                required
                                prop:value=move || username.get()
                                on:input=move |ev| username.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="input-field">
                            <input
                                type="password"
                                class="input-text"
                                placeholder="Enter password"
                                required
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    {move || response_msg.get().map(|(msg, success)| {
                        view! {
                            <div class="response_msg" class:success=success class:error=!success>
                                {msg}
                            </div>
                        }
                    })}
                    <button
                        class="save-btn full-width-btn"
                        class:btn-blue=can_submit
                        prop:disabled=move || !can_submit()
                    >
                        "Submit"
                    </button>
                </form>
                <button class="button btn-transparent" on:click=move |_| on_back()>
                    "Back"
                </button>
            </div>
        </div>
    }
}
```

#### 3.4 Change Password Panel (`src/components/settings/change_password.rs`)

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::settings::update_password;
use crate::components::password_strength::PasswordStrength;
use crate::components::toast::use_toast;

#[component]
pub fn ChangePasswordPanel(on_back: impl Fn() + 'static) -> impl IntoView {
    let toast = use_toast();
    let old_password = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    let passwords_match = move || {
        let np = new_password.get();
        let cp = confirm_password.get();
        !np.is_empty() && np == cp
    };

    let password_valid = move || {
        let pw = new_password.get();
        pw.len() >= 8
            && pw.chars().any(|c| c.is_uppercase())
            && pw.chars().any(|c| c.is_ascii_digit())
            && pw.chars().any(|c| "!@#$%^&*(),.?\":{}|<>".contains(c))
    };

    let can_submit = move || {
        !old_password.get().is_empty()
            && password_valid()
            && passwords_match()
            && !is_submitting.get()
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !can_submit() { return; }

        is_submitting.set(true);
        let new_pw = new_password.get().clone();
        let old_pw = old_password.get().clone();

        spawn_local(async move {
            match update_password(new_pw, old_pw).await {
                Ok(()) => {
                    toast.success("Password updated!");
                    old_password.set(String::new());
                    new_password.set(String::new());
                    confirm_password.set(String::new());
                    response_msg.set(Some(("Password changed successfully.".to_string(), true)));
                }
                Err(e) => {
                    response_msg.set(Some((e.to_string(), false)));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="profile-widget active">
            <div class="edit-profile change-password">
                <h2 class="section_heading">"Change password"</h2>
                <form class="form-container" on:submit=on_submit>
                    <div class="profile-fields">
                        <div class="input-field">
                            <img class="password-icon" src="/svg/lock1.svg" alt=""/>
                            <input
                                type="password"
                                class="input-text"
                                placeholder="Enter old password"
                                required
                                prop:value=move || old_password.get()
                                on:input=move |ev| old_password.set(event_target_value(&ev))
                            />
                        </div>

                        // New password with strength indicator
                        <div class="password-component">
                            <div class="input-field">
                                <img class="password-icon" src="/svg/lock1.svg" alt=""/>
                                <input
                                    type="password"
                                    class="input-text"
                                    placeholder="New password"
                                    required
                                    prop:value=move || new_password.get()
                                    on:input=move |ev| new_password.set(event_target_value(&ev))
                                />
                            </div>
                            <PasswordStrength password=new_password/>
                        </div>

                        // Confirm password
                        <div class="input-field">
                            <img class="password-icon" src="/svg/lock1.svg" alt=""/>
                            <input
                                type="password"
                                class="input-text"
                                placeholder="Confirm password"
                                required
                                prop:value=move || confirm_password.get()
                                on:input=move |ev| confirm_password.set(event_target_value(&ev))
                            />
                        </div>

                        // Match validation
                        <Show when=move || !confirm_password.get().is_empty() && !passwords_match()>
                            <div class="validationMessage notvalid">"Passwords do not match"</div>
                        </Show>
                    </div>

                    {move || response_msg.get().map(|(msg, success)| {
                        view! {
                            <div class="response_msg" class:success=success class:error=!success>
                                {msg}
                            </div>
                        }
                    })}

                    <button
                        class="save-btn full-width-btn"
                        class:btn-blue=can_submit
                        prop:disabled=move || !can_submit()
                    >
                        "Submit"
                    </button>
                </form>
                <button class="button btn-transparent" on:click=move |_| on_back()>
                    "Back"
                </button>
            </div>
        </div>
    }
}
```

#### 3.5 Change Email Panel (`src/components/settings/change_email.rs`)

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::settings::update_email;
use crate::components::toast::use_toast;

#[component]
pub fn ChangeEmailPanel(on_back: impl Fn() + 'static) -> impl IntoView {
    let toast = use_toast();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    let email_valid = move || {
        let e = email.get();
        // Basic email validation
        e.contains('@') && e.contains('.') && e.len() >= 5
    };

    let can_submit = move || {
        email_valid()
            && !password.get().is_empty()
            && !is_submitting.get()
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !can_submit() { return; }

        is_submitting.set(true);
        let new_email = email.get().trim().to_string();
        let pw = password.get().clone();

        spawn_local(async move {
            match update_email(new_email, pw).await {
                Ok(()) => {
                    toast.success("Email updated!");
                    email.set(String::new());
                    password.set(String::new());
                    response_msg.set(Some(("Email changed successfully.".to_string(), true)));
                }
                Err(e) => {
                    response_msg.set(Some((e.to_string(), false)));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="profile-widget active">
            <div class="edit-profile change-email">
                <h2 class="section_heading">"Change e-mail"</h2>
                <form class="form-container" on:submit=on_submit>
                    <div class="profile-fields">
                        <div class="input-field">
                            <input
                                type="email"
                                class="input-text"
                                placeholder="Enter new e-mail"
                                required
                                prop:value=move || email.get()
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="input-field">
                            <input
                                type="password"
                                class="input-text"
                                placeholder="Enter password"
                                required
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    {move || response_msg.get().map(|(msg, success)| {
                        view! {
                            <div class="response_msg" class:success=success class:error=!success>
                                {msg}
                            </div>
                        }
                    })}
                    <button
                        class="save-btn full-width-btn"
                        class:btn-blue=can_submit
                        prop:disabled=move || !can_submit()
                    >
                        "Submit"
                    </button>
                </form>
                <button class="button btn-transparent" on:click=move |_| on_back()>
                    "Back"
                </button>
            </div>
        </div>
    }
}
```

### Phase 4: Content Settings Component

#### 4.1 Content Settings (`src/components/settings/content.rs`)

```rust
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::posts::get_user_info;
use crate::api::settings::update_content_preferences;
use crate::components::toast::use_toast;

/// Content Settings tab — reported content toggle.
#[component]
pub fn ContentSettings() -> impl IntoView {
    let toast = use_toast();
    let is_lenient = RwSignal::new(false);
    let is_loading = RwSignal::new(true);
    let show_confirm = RwSignal::new(false);
    let pending_state = RwSignal::new(false);
    let show_success = RwSignal::new(false);
    let success_message = RwSignal::new(String::new());

    // Load current preference
    let user_info_resource = Resource::new(
        || (),
        |_| async move {
            // getUserInfo returns current preferences
            // We need the authenticated user's ID from auth context
            get_user_info("self".to_string()).await
        }
    );

    Effect::new(move |_| {
        if let Some(Ok(info)) = user_info_resource.get() {
            if let Some(prefs) = info.user_preferences {
                is_lenient.set(
                    prefs.content_filtering_severity_level == "MYGRANDMAHATES"
                );
            }
            is_loading.set(false);
        }
    });

    let on_toggle = move |_| {
        pending_state.set(!is_lenient.get());
        show_confirm.set(true);
    };

    let on_confirm = move || {
        show_confirm.set(false);
        let new_level = if pending_state.get() {
            "MYGRANDMAHATES"
        } else {
            "MYGRANDMALIKES"
        };

        spawn_local(async move {
            match update_content_preferences(new_level.to_string()).await {
                Ok(()) => {
                    is_lenient.set(pending_state.get());
                    if pending_state.get() {
                        success_message.set("Content restored. Reported posts are now visible in your feed.".to_string());
                    } else {
                        success_message.set("Hidden successfully. Reported posts have been removed from your feed.".to_string());
                    }
                    show_success.set(true);
                }
                Err(e) => {
                    toast.error(&format!("Failed to update: {}", e));
                }
            }
        });
    };

    view! {
        <div id="content-settings" class="setting-content">
            <div class="reported-content-btn">
                <span class="md_font_size">"Reported content view"</span>
                <label class="switch">
                    <input
                        type="checkbox"
                        prop:checked=move || is_lenient.get()
                        prop:disabled=move || is_loading.get()
                        on:click=on_toggle
                    />
                    <span>
                        <span class="label-on md_font_size">"on"</span>
                        <span class="label-off md_font_size">"off"</span>
                    </span>
                </label>
            </div>

            // Confirmation dialog
            <Show when=move || show_confirm.get()>
                <ContentConfirmDialog
                    is_enabling=pending_state.get()
                    on_confirm=on_confirm
                    on_cancel=move || show_confirm.set(false)
                />
            </Show>

            // Success modal
            <Show when=move || show_success.get()>
                <ContentSuccessModal
                    message=success_message.get()
                    on_close=move || show_success.set(false)
                />
            </Show>
        </div>
    }
}

#[component]
fn ContentConfirmDialog(
    is_enabling: bool,
    on_confirm: impl Fn() + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let (title, description) = if is_enabling {
        ("Show reported content?", "You're about to see all reported posts in your feed.")
    } else {
        ("Hide all reported posts?", "You're about to hide all reported posts from your feed.")
    };

    view! {
        <div class="modal-overlay">
            <div class="logOut-pop">
                <i class="peer-icon peer-icon-warning"/>
                <h3 class="xxl_font_size bold">{title}</h3>
                <p class="xl_font_size">{description}</p>
                <div class="button-row">
                    <button class="btn-white" on:click=move |_| on_cancel()>"Cancel"</button>
                    <button class="btn-blue" on:click=move |_| on_confirm()>"Confirm"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ContentSuccessModal(
    message: String,
    on_close: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="modal-overlay">
            <div class="logOut-pop">
                // Checkmark SVG
                <svg xmlns="http://www.w3.org/2000/svg" width="80" height="80" viewBox="0 0 186 186" fill="none">
                    <path d="M93 186C80.135 186 68.045 183.559 56.73 178.676..." fill="#AAFF67"/>
                </svg>
                <p class="xl_font_size">{message}</p>
                <div class="button-row">
                    <button class="btn-blue" on:click=move |_| on_close()>"OK"</button>
                </div>
            </div>
        </div>
    }
}
```

### Phase 5: Stub Tabs & Account Deactivation

#### 5.1 Notification & Preferences Stubs

```rust
// src/components/settings/notification.rs
#[component]
pub fn NotificationSettings() -> impl IntoView {
    view! {
        <div id="notification-settings" class="setting-content">
            <p class="md_font_size txt-color-gray">"Notification settings coming soon."</p>
        </div>
    }
}

// src/components/settings/preferences.rs
#[component]
pub fn PreferencesSettings() -> impl IntoView {
    view! {
        <div id="prefrences" class="setting-content">
            <p class="md_font_size txt-color-gray">"Preferences coming soon."</p>
        </div>
    }
}
```

#### 5.2 Account Deactivation Flow

```rust
// Added to settings menu or as a sub-panel
#[component]
fn DeactivateAccountPanel() -> impl IntoView {
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let show_confirm = RwSignal::new(false);
    let auth = use_auth_context();

    let on_deactivate = move || {
        if password.get().is_empty() || is_submitting.get() { return; }
        is_submitting.set(true);
        let pw = password.get().clone();

        spawn_local(async move {
            match delete_account(pw).await {
                Ok(()) => {
                    // Account deleted — log out and redirect
                    auth.logout_action.dispatch(());
                }
                Err(e) => {
                    leptos::logging::error!("Deactivation failed: {:?}", e);
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="deactivate-panel">
            <h2 class="section_heading">"Deactivate Account"</h2>
            <p class="md_font_size txt-color-gray">
                "This will permanently deactivate your account. This action cannot be undone."
            </p>
            <div class="input-field">
                <input
                    type="password"
                    class="input-text"
                    placeholder="Enter your password to confirm"
                    prop:value=move || password.get()
                    on:input=move |ev| password.set(event_target_value(&ev))
                />
            </div>
            <button
                class="btn-red-transparent full-width-btn"
                prop:disabled=move || password.get().is_empty() || is_submitting.get()
                on:click=move |_| show_confirm.set(true)
            >
                "Deactivate Account"
            </button>

            <Show when=move || show_confirm.get()>
                <div class="modal-overlay">
                    <div class="logOut-pop">
                        <p class="xl_font_size bold">"Are you sure? This cannot be undone."</p>
                        <div class="button-row">
                            <button class="btn-white" on:click=move |_| show_confirm.set(false)>"Cancel"</button>
                            <button class="btn-red-transparent" on:click=move |_| on_deactivate()>"Deactivate"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
```

---

## File Structure

```
src/
├── api/
│   ├── settings.rs              # NEW: Settings mutations (profile, password, email, etc.)
│   ├── graphql.rs               # MODIFY: Add mutation strings + response types
│   └── mod.rs                   # MODIFY: Add settings module + re-exports
├── components/
│   └── settings/
│       ├── mod.rs               # NEW: Module exports
│       ├── menu.rs              # NEW: Tab navigation + logout modal
│       ├── profile.rs           # NEW: Profile settings (avatar, bio, username)
│       ├── image_modal.rs       # NEW: Image upload preview modal
│       ├── change_username.rs   # NEW: Username change form
│       ├── change_password.rs   # NEW: Password change form
│       ├── change_email.rs      # NEW: Email change form
│       ├── content.rs           # NEW: Content filtering toggle
│       ├── notification.rs      # NEW: Notification stub
│       ├── preferences.rs       # NEW: Preferences stub
│       └── deactivate.rs        # NEW: Account deactivation
├── models/
│   ├── settings.rs              # NEW: Settings response types
│   └── mod.rs                   # MODIFY: Add settings module
├── pages/
│   ├── settings.rs              # NEW: Settings page
│   └── mod.rs                   # MODIFY: Add settings page export
├── app.rs                       # MODIFY: Add /settings route
└── style/
    └── settings.scss            # NEW: Settings-specific styles
```

---

## Testing Plan

### Unit Tests

- [ ] `UpdateResponse::is_success()` returns correct values
- [ ] Password validation logic (length, uppercase, digit, special)
- [ ] Email validation logic
- [ ] Username validation (format, length)
- [ ] Biography encoding to base64 data URI

### Integration Tests

- [ ] Profile image update mutation returns success
- [ ] Biography update mutation returns success
- [ ] Username change with correct password succeeds
- [ ] Username change with wrong password fails
- [ ] Password change with matching confirmation succeeds
- [ ] Password change with weak password fails
- [ ] Email change with valid email succeeds
- [ ] Content preferences toggle updates correctly
- [ ] Deactivate account with correct password succeeds
- [ ] All mutations fail correctly when not authenticated

### E2E Tests (Playwright)

- [ ] Navigate to `/settings` while authenticated
- [ ] Verify redirect to login when unauthenticated
- [ ] Tab navigation switches content panels
- [ ] Upload profile picture and see preview
- [ ] Edit biography and save
- [ ] Change username with correct password
- [ ] Change password with strength indicator
- [ ] Change email with password confirmation
- [ ] Toggle content filtering with confirmation
- [ ] Logout flow with confirmation modal
- [ ] Verify form validation messages appear
- [ ] Mobile responsive layout

---

## Acceptance Criteria

1. **Auth Guard**
   - [ ] Unauthenticated users redirected to `/login`
   - [ ] Redirect back to `/settings` after login

2. **Tab Navigation**
   - [ ] All four tabs clickable and switch content
   - [ ] Active tab highlighted with arrow indicator
   - [ ] Tab state preserved during form interactions

3. **Profile Settings**
   - [ ] Current avatar displayed on load
   - [ ] File picker limited to images
   - [ ] Image preview with zoom slider
   - [ ] Biography loaded from API and editable
   - [ ] Save updates both avatar and bio in parallel
   - [ ] Success/error feedback shown
   - [ ] Username displayed with @ prefix

4. **Username Change**
   - [ ] Sub-panel opens on "Change" click
   - [ ] Requires new username + password
   - [ ] Submit disabled until fields filled
   - [ ] Success message on update
   - [ ] Back button returns to main panel

5. **Password Change**
   - [ ] Old password, new password, confirm password fields
   - [ ] Password strength bar updates live
   - [ ] "Passwords do not match" shown when mismatched
   - [ ] Submit disabled until all requirements met
   - [ ] Success clears fields and shows message

6. **Email Change**
   - [ ] Email format validation
   - [ ] Requires password confirmation
   - [ ] Success message on update

7. **Content Settings**
   - [ ] Toggle reflects current preference
   - [ ] Confirmation dialog shown before change
   - [ ] Success modal after API update
   - [ ] Visual toggle state updates

8. **Logout**
   - [ ] Confirmation modal appears
   - [ ] Cancel closes modal
   - [ ] Confirm clears session and redirects to login

9. **Account Deactivation**
   - [ ] Requires password confirmation
   - [ ] Second confirmation modal ("cannot be undone")
   - [ ] On success, logs out and redirects

10. **Responsive**
    - [ ] Desktop: side-by-side menu and content
    - [ ] Mobile: stacked layout
    - [ ] All modals work on mobile viewports

---

## Dependencies

- Existing: `leptos`, `leptos_router`, `leptos_meta`
- Existing: `serde`, `serde_json`
- Existing: Auth context, GraphQL client, toast system
- Existing: `PasswordStrength` component (reuse from registration)
- Existing: `AuthGuard` component
- New: `base64` crate (for biography encoding — check if already in deps)
- New: `futures` crate (for `join!` macro — check if already in deps)

---

## Estimated Timeline

| Phase | Description | Days |
|-------|-------------|------|
| Phase 1 | Models & API Layer | 1 |
| Phase 2 | Settings Page & Tab Nav | 1 |
| Phase 3 | Profile Settings Components | 2 |
| Phase 4 | Content Settings | 0.5 |
| Phase 5 | Stubs & Deactivation | 0.5 |
| Styling | SCSS port from legacy CSS | 1 |
| Testing | Unit, Integration, E2E | 1.5 |
| **Total** | | **7.5 days** |

---

## Changelog

### 2026-04-14
- Initial planning document created
- Analyzed legacy implementation across 6 PHP templates and 5 JS files
- Documented all 7 backend mutations with response codes
- Defined 5-phase implementation plan with Rust component code
- Mapped file structure (12 new files, 4 modified)
- Created testing and acceptance criteria
