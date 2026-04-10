# Registration Flow — Leptos Migration Plan

## Overview

This document outlines 15 incremental steps to convert the Peer Network **registration system** (`register.php` + `js/register/register.js`) from the current PHP/vanilla-JS stack to Leptos (Rust). Each step has a concrete testing outcome. The plan assumes a **mock Peer backend** is built first so that all steps can be validated without depending on the live GraphQL API.

---

## Prerequisites

- Rust toolchain installed (`rustup`)
- `cargo-leptos` CLI installed
- Node.js (for running the mock backend)
- Familiarity with the study document: `docs/leptos-rewrite-study.md`

---

## Step 0 — Mock Peer Backend

**Goal:** Stand up a lightweight mock GraphQL server that simulates the three registration-related mutations (`verifyReferralString`, `register`, `verifyAccount`) with exact field names, response envelopes, and response codes — so all subsequent steps can be developed and tested offline.

**Detailed plan:** [step0-mock-backend.md](step0-mock-backend.md)

**Testing outcome:** `curl -X POST http://localhost:4000/graphql` with each of the three mutations returns correctly shaped JSON responses matching the real Peer API.

---

## Step 1 — Scaffold Leptos Project

**Goal:** Initialize the `cargo-leptos` project in a new `peer-web/` directory alongside the existing PHP app.

**Detailed plan:** [step1-scaffold-leptos-project.md](step1-scaffold-leptos-project.md)

### Actions

- `cargo leptos new peer-web` (Axum template)
- Configure `Cargo.toml` with SSR + CSR features
- Set `GRAPHQL_ENDPOINT` env var to `http://localhost:4000/graphql` (mock)
- Verify `cargo leptos watch` compiles and serves the default page

**Testing outcome:** Visiting `http://localhost:3000` in a browser renders the Leptos welcome page. The terminal shows no compilation errors. `cargo test` passes the default test.

---

## Step 2 — Project Structure & Shared Types

**Goal:** Create the module skeleton and define the Rust types that mirror the GraphQL registration schema.

**Detailed plan:** [step2-project-structure-shared-types.md](step2-project-structure-shared-types.md)

### Files to create

```
src/
├── api/mod.rs
├── api/graphql.rs          ← generic query/mutation helper
├── models/mod.rs
├── models/user.rs          ← RegistrationInput, ReferralResponse, RegisterResponse, VerifyResponse
├── components/mod.rs
├── pages/mod.rs
├── pages/register.rs       ← stub RegisterPage component
└── state/mod.rs
```

### Types (in `models/user.rs`)

```rust
pub struct RegistrationInput {
    pub email: String,
    pub password: String,
    pub username: String,
    pub pkey: Option<String>,
    pub referral_uuid: String,
}

pub struct ReferralVerifyResponse {
    pub status: String,
    pub response_code: String,
    pub affected_rows: Option<Vec<ReferralUser>>,
}

pub struct RegisterResponse {
    pub status: String,
    pub response_code: String,
    pub userid: Option<String>,
}
```

**Testing outcome:** `cargo check` succeeds. A unit test in `models/user.rs` deserializes a sample JSON fixture (`fixtures/register_success.json`) into `RegisterResponse` without error.

---

## Step 3 — GraphQL Client Module

**Goal:** Implement a reusable async GraphQL client in `src/api/graphql.rs` that sends queries/mutations to the backend and deserializes typed responses.

### Implementation

- Generic `pub async fn mutate<V, T>(query, variables, token) -> Result<T, ApiError>`
- Uses `reqwest` on the server side
- Reads endpoint from env `GRAPHQL_ENDPOINT`
- Handles `{ data, errors }` envelope

**Testing outcome:** An integration test (`#[tokio::test]`) calls `mutate` against the running mock backend with the `VerifyReferralString` mutation and asserts `status == "success"`.

---

## Step 4 — Server Functions for Registration

**Goal:** Create Leptos server functions that wrap the three GraphQL mutations. These run on the Axum server and are callable from client-side WASM.

**Detailed plan:** [step4-server-functions.md](step4-server-functions.md)

### Server functions

```rust
#[server(VerifyReferral, "/api")]
pub async fn verify_referral(referral_string: String) -> Result<ReferralVerifyResponse, ServerFnError>

#[server(RegisterUser, "/api")]
pub async fn register_user(input: RegistrationInput) -> Result<RegisterResponse, ServerFnError>

#[server(VerifyAccount, "/api")]
pub async fn verify_account(userid: String) -> Result<VerifyResponse, ServerFnError>
```

**Testing outcome:** A test calls `verify_referral("85d5f836-b1f5-4c4e-9381-1b058e13df93".into())` via the server function HTTP endpoint (`POST /api/verify_referral`) and receives a success response. A second test sends an invalid string and asserts an error.

---

## Step 5 — Router & Page Shell

**Goal:** Set up the Leptos router with the `/register` route and a minimal `RegisterPage` component that renders an empty multi-step container.

**Detailed plan:** [step5-router-and-page-shell.md](step5-router-and-page-shell.md)

### Implementation

- Add route `<Route path="/register" view=RegisterPage />` in `app.rs`
- `RegisterPage` renders a `<div class="container">` with step placeholders
- Import existing `css/login-register.css` into the project

**Testing outcome:** Navigating to `http://localhost:3000/register` renders a page with the "container" div. View-source confirms SSR HTML is present (not blank WASM shell).

---

## Step 6 — Step 1 UI: Referral Code Entry

**Goal:** Build the first registration step — the referral code input form with client-side UUID validation.

**Detailed plan:** [step6-referral-code-ui.md](step6-referral-code-ui.md)

### Implementation

- Reactive signal `referral_code: RwSignal<String>`
- Client-side regex validation (`/^[0-9a-f]{8}-...$/i`)
- Validation message display (valid/invalid styling)
- "Don't have a code?" link to show default referral sub-step
- URL parameter prefill (`?ref=...`)

**Testing outcome:** 
1. Entering a valid UUID shows the green check icon and enables the "Verify Code" button.
2. Entering "abc123" shows the error message "Hmm… that referral code doesn't seem to work."
3. Loading `/register?ref=85d5f836-b1f5-4c4e-9381-1b058e13df93` auto-fills the input.

---

## Step 7 — Referral Verification (Server Round-Trip)

**Goal:** Wire the "Verify Code" button to the `verify_referral` server function and handle success/error responses.

**Detailed plan:** [step7-referral-verification-server-roundtrip.md](step7-referral-verification-server-roundtrip.md)

### Implementation

- `create_action` wrapping `verify_referral`
- Loading state on button (disabled + spinner)
- On success → advance to step 2
- On error → show toast with user-friendly message from response code

**Testing outcome:**
1. Submitting a valid referral code against the mock backend transitions to step 2.
2. Submitting an unknown code shows an error toast and stays on step 1.
3. While the request is in-flight, the button is disabled and shows a loading indicator.

---

## Step 8 — Step 2 UI: Registration Form Fields

**Goal:** Build the registration form with email, username, password, confirm-password, and checkbox fields — all with real-time client-side validation.

### Implementation

- Reactive signals for each field: `email`, `username`, `password`, `confirm_password`
- Validator functions matching the existing JS regex rules:
  - Email: `/^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/`
  - Username: `/^[a-zA-Z0-9_-]{3,23}$/`
  - Password: length ≥ 8, lowercase, uppercase, digit (required); special char (bonus)
  - Confirm password: matches password
- Password strength meter (weak → weak2 → medium → strong → excellent)
- Password visibility toggle
- Privacy policy + EULA checkboxes

**Testing outcome:**
1. Typing `user@example.com` shows valid state; typing `user@` shows invalid state.
2. Username `ab` shows invalid; `peer_user` shows valid.
3. Password `Abcd1234` shows "strong"; `abcd` shows "very weak".
4. Mismatched confirm password shows "Passwords do not match".
5. Submitting with unchecked checkboxes shows "Please accept both the Privacy Policy and EULA".

---

## Step 9 — Registration Submission (Server Round-Trip)

**Goal:** Wire the "Register" button to the `register_user` server function and handle all response paths.

**Detailed plan:** [step9-registration-submission.md](step9-registration-submission.md)

### Implementation

- Full form validation gate before submission
- `create_action` wrapping `register_user`
- On success (`10601`) → call `verify_account`, advance to step 3
- On duplicate email (`30601`) → highlight email field with backend error message
- On other errors → show toast

**Testing outcome:**
1. Submitting valid data against the mock backend transitions to step 3 (success screen).
2. Registering with an already-used email shows "This email is already registered" on the email field.
3. Network error (mock backend stopped) shows a generic error toast.

---

## Step 10 — Step 3 UI: Success Confirmation

**Goal:** Build the success/welcome screen shown after registration completes.

**Detailed plan:** [step10-success-confirmation-ui.md](step10-success-confirmation-ui.md)

### Implementation

- Display "Registration successful! Welcome to peer!" message
- Store new user email in session storage (for login page auto-fill)
- Provide a "Go to Login" link navigating to `/login`
- Screen reader announcement via `aria-live` region

**Testing outcome:**
1. After a successful registration, step 3 is visible with the success message.
2. `sessionStorage.getItem('newUserEmail')` returns the registered email.
3. Clicking "Go to Login" navigates to `/login`.

---

## Step 11 — Navigation & Back Button

**Goal:** Implement the multi-step navigation logic — back button behaviour, step transitions, and browser history integration.

### Implementation

- `current_step: RwSignal<u8>` controlling which step div is `.active`
- Back button returns to previous step (step 2 → step 1)
- Back button on step 1 navigates to `/login` (external link)
- Back button hidden on step 3 (success)
- Focus management: first focusable element in new step receives focus

**Testing outcome:**
1. On step 2, clicking Back returns to step 1 with the referral code preserved.
2. On step 1, clicking Back navigates to `/login`.
3. On step 3, the back button is not visible.
4. After step transition, the first input in the new step has focus.

---

## Step 12 — Toast Notification Component

**Goal:** Build a reusable toast/notification component matching the existing behaviour and remap response codes to user-friendly messages.

**Detailed plan:** [step12-toast-notification-component.md](step12-toast-notification-component.md)

### Implementation

- `<Toast message=... toast_type=... />` component with auto-dismiss (3s)
- Types: `info`, `success`, `error`
- `aria-live="assertive"` for screen reader support
- Utility function `user_friendly_msg(code: &str) -> &str` mapping `response-codes.json` entries

**Testing outcome:**
1. Triggering a success toast shows a green banner that auto-dismisses after 3 seconds.
2. `user_friendly_msg("10601")` returns "Registration successful! Please check your email to verify your account."
3. `user_friendly_msg("30601")` returns the duplicate-email message.

---

## Step 13 — Accessibility & Screen Reader Support

**Goal:** Ensure the registration flow meets WCAG 2.1 AA, matching or exceeding the current accessibility features.

**Detailed plan:** [step13-accessibility-screen-reader-support.md](step13-accessibility-screen-reader-support.md)

### Implementation

- `aria-describedby` on all inputs linking to validation messages and help text
- `aria-invalid="true"` on fields with errors
- `role="alert"` on validation message containers
- Step announcer (`aria-live="polite"`) announces step changes
- Keyboard navigation: Tab order is logical, Enter submits forms, Escape closes modals

**Testing outcome:**
1. `axe-core` audit of `/register` reports zero critical or serious violations.
2. Navigating the entire flow using only keyboard (Tab, Enter, Shift+Tab) works without traps.
3. VoiceOver (macOS) announces step transitions and validation errors.

---

## Step 14 — CSS & Visual Parity

**Goal:** Achieve pixel-level visual parity with the existing PHP registration page.

**Detailed plan:** [step14-css-visual-parity.md](step14-css-visual-parity.md)

### Implementation

- Import `css/login-register.css` unchanged as a starting point
- Import `fonts/font-poppins/stylesheet.css` and `fonts/peer-icon-font/css/peer-network.css`
- Map CSS classes used in `register.php` to the Leptos `view!` markup
- Verify responsive behaviour (mobile, tablet, desktop)
- Ensure password strength meter colours and animations match
- Phone mockup image and logo SVGs display correctly

**Testing outcome:**
1. Side-by-side screenshot comparison (Percy, BackstopJS, or manual) of the PHP and Leptos versions at 375px, 768px, and 1440px widths show no visible regressions.
2. All Peer icon font glyphs (`peer-icon-referral`, `peer-icon-envelope`, etc.) render correctly.
3. Password strength meter transitions through all 5 colour states.

---

## Step 15 — End-to-End Integration Test Suite

**Goal:** Create an automated E2E test suite that exercises the full registration happy path and key error paths against the mock backend.

### Implementation

Use Playwright (or `fantoccini` for Rust-native):

```
tests/e2e/
├── registration.spec.ts
├── helpers/
│   └── mock-server.ts      ← starts/stops mock backend
└── playwright.config.ts
```

### Test cases

| # | Test | Assertion |
|---|------|-----------|
| 1 | Happy path: valid referral → valid form → success | Step 3 visible, session storage has email |
| 2 | Invalid referral code format | Error message on referral field |
| 3 | Unknown referral code (server rejects) | Error toast displayed |
| 4 | Duplicate email registration | Email field shows backend error |
| 5 | Weak password rejected | Password requirements shown |
| 6 | Mismatched confirm password | "Passwords do not match" visible |
| 7 | Unchecked checkboxes | Checkbox error message visible |
| 8 | URL param prefill (`?ref=...`) | Referral input auto-filled |
| 9 | Back navigation preserves state | Return to step 1 keeps referral code |
| 10 | Auto-redirect if already logged in | Redirect to `/dashboard` |

**Testing outcome:** `npx playwright test` (or `cargo test --test e2e`) passes all 10 test cases in < 30 seconds against the mock backend. CI pipeline runs these on every PR.

---

## Summary

| Step | Milestone | Depends on |
|------|-----------|------------|
| 0 | Mock backend running | — |
| 1 | Leptos project compiles & serves | 0 |
| 2 | Shared types & module skeleton | 1 |
| 3 | GraphQL client talks to mock | 0, 2 |
| 4 | Server functions callable | 3 |
| 5 | `/register` route renders | 1 |
| 6 | Referral UI with client validation | 5 |
| 7 | Referral server verification | 4, 6 |
| 8 | Registration form with validation | 5 |
| 9 | Registration server submission | 4, 8 |
| 10 | Success screen | 9 |
| 11 | Step navigation & back button | 6, 8, 10 |
| 12 | Toast component & response codes | 7, 9 |
| 13 | Accessibility audit pass | 6–12 |
| 14 | Visual parity with PHP version | 6–12 |
| 15 | E2E test suite green | 0–14 |

Steps 6–8 can be worked on in parallel. Steps 13–14 can also proceed in parallel once the UI steps are complete.
