# Step 9 — Registration Submission (Server Round-Trip)

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Wire the "Register" button to the `register_user` server function and handle all response paths — including success, validation errors, duplicate email, and network failures. This step completes the data flow from client form submission through server-side GraphQL mutation to UI feedback.

---

## 9.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 8 complete | `cargo leptos build` | Compiles; registration form renders with all fields |
| Server functions working | `cargo test --features ssr --test server_functions` | All tests pass including `register_user` |
| Mock backend running | `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` | Returns `{"data":{"_health":true}}` |
| Form signals exist | Check `src/pages/register.rs` | `email`, `username`, `password`, `confirm_password` signals defined |
| Client-side validation working | Manual test in browser | Invalid inputs show error styling |
| Toast component available | Check `src/components/toast.rs` | `use_toast()` hook and `ToastType` enum exist |
| Step 7 pattern understood | Review step 7 implementation | Familiar with `Action` + `Effect` pattern |

---

## 9.2 — Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           RegisterPage Component                                 │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │ Form Signals (from Step 8)                                                  ││
│  │ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌───────────────────────┐││
│  │ │email         │ │username      │ │password      │ │confirm_password       │││
│  │ │RwSignal<Str> │ │RwSignal<Str> │ │RwSignal<Str> │ │RwSignal<String>       │││
│  │ └──────────────┘ └──────────────┘ └──────────────┘ └───────────────────────┘││
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │ Validation State Signals                                                    ││
│  │ ┌──────────────────┐ ┌──────────────────┐ ┌────────────────────────────────┐││
│  │ │email_error       │ │username_error    │ │checkboxes_valid                │││
│  │ │RwSignal<Option>  │ │RwSignal<Option>  │ │Memo<bool>                      │││
│  │ └──────────────────┘ └──────────────────┘ └────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │ register_action = Action::new(|input| register_user(input))                 ││
│  │                                                                             ││
│  │   .pending() → bool        (is request in-flight?)                          ││
│  │   .value()   → Option<Result<RegisterResponse, ServerFnError>>              ││
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │ "Register" Button                                                           ││
│  │                                                                             ││
│  │   disabled when: pending() || !form_valid()                                 ││
│  │   on:click → validate_and_submit()                                          ││
│  │   shows spinner + "Creating account..." when pending()                      ││
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │ Effect: Handle register_action Result                                       ││
│  │                                                                             ││
│  │   match response.response_code:                                             ││
│  │     "10601" → call verify_account() → set current_step(3)                   ││
│  │     "30601" → set email_error("Email already registered")                   ││
│  │     "31007" → toast("Invalid referral") - shouldn't happen at this step     ││
│  │     "40601" → toast("Registration failed, try again")                       ││
│  │     other   → toast(user_friendly_msg(code))                                ││
│  └─────────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ POST /api/register_user
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          Axum Server (SSR)                                       │
│                                                                                 │
│  register_user(input: RegistrationInput) → Result<RegisterResponse>             │
│       │                                                                         │
│       ├─ Server-side validation (defense in depth)                              │
│       │     - UUID format for referral_uuid                                     │
│       │     - Email format                                                      │
│       │     - Password complexity                                               │
│       │     - Username format                                                   │
│       │                                                                         │
│       └─ graphql::mutate(REGISTER_MUTATION, variables)                          │
└─────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ POST http://localhost:4000/graphql
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          Mock Backend / Production API                           │
│                                                                                 │
│  mutation Register($input: RegistrationInput!) {                                 │
│    register(input: $input) {                                                     │
│      status                                                                      │
│      ResponseCode                                                                │
│      userid                                                                      │
│    }                                                                             │
│  }                                                                               │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Validate before dispatch | Prevent unnecessary network calls; provide instant feedback |
| Show field-level backend errors | `30601` (duplicate email) maps to the email field, not a toast |
| Chain `verify_account` on success | Original JS calls `verifyUser2()` after successful registration |
| Store email in session storage | Allows login page to pre-fill the email for continuity |
| Separate validation state from form state | Allows backend errors to coexist with client validation |
| Gate submission on all validations | Button disabled until email, username, password, checkboxes all valid |

---

## 9.3 — Response Code Mapping

The following response codes are relevant to the registration submission flow:

| Code | Meaning | UI Handling |
|------|---------|-------------|
| `10601` | Registration successful | Call `verify_account`, store email, advance to step 3, success toast |
| `30301` | Missing required fields | Show toast — shouldn't happen with client validation |
| `30601` | Email already registered | Set `email_error` signal; highlight email field |
| `31007` | Invalid referral UUID | Show toast — shouldn't happen if step 7 succeeded |
| `30202` | Invalid username format | Set `username_error` signal; highlight username field |
| `30103` | Invalid input format | Show toast with generic message |
| `40601` | Server failed to register | Show toast: "Registration failed. Please try again." |
| `40602` | Failed to generate user ID | Show toast: "Registration failed. Please try again." |

### User-Friendly Messages

Leverage the existing `response-codes.json` mapping via `user_friendly_msg()`:

```rust
// In src/utils/response_codes.rs
pub fn user_friendly_msg(code: &str) -> &'static str {
    match code {
        "10601" => "Registration successful! Please check your email to verify your account.",
        "30601" => "Email already registered. Use a different one.",
        "30202" => "Invalid username format. Use 3-23 characters: letters, numbers, underscores, or hyphens.",
        "31007" => "Invalid referral code. Please check and try again.",
        "40601" => "We couldn't complete your registration. Please try again or contact support.",
        "40602" => "We're having trouble creating your account. Please try again.",
        _ => "An unexpected error occurred. Please try again.",
    }
}
```

---

## 9.4 — Implementation

### 9.4.1 — Add Imports

At the top of `src/pages/register.rs`, add:

```rust
use crate::api::registration::{register_user, verify_account};
use crate::models::user::{RegistrationInput, RegisterResponse, VerifyResponse};
use crate::components::toast::{ToastType, use_toast};
use crate::utils::response_codes::user_friendly_msg;
```

### 9.4.2 — Add Backend Error Signals

Inside the `RegisterPage` component, add signals to track backend-specific validation errors. These are separate from the client-side validation state so backend errors can be displayed even when client validation passes.

```rust
#[component]
pub fn RegisterPage() -> impl IntoView {
    // Existing signals from Step 6 & 8
    let referral_code = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let privacy_checked = RwSignal::new(false);
    let eula_checked = RwSignal::new(false);
    let current_step = RwSignal::new(1u8);
    
    // NEW: Backend error signals (distinct from client validation)
    // None = no backend error, Some(msg) = backend rejected this field
    let email_backend_error = RwSignal::new(None::<String>);
    let username_backend_error = RwSignal::new(None::<String>);
    
    // Toast context
    let toast = use_toast();
    
    // ... rest of component
}
```

### 9.4.3 — Create Registration Action

Create the server action for registration:

```rust
    // Create the server action for registration
    let register_action = Action::new(move |input: &RegistrationInput| {
        let input = input.clone();
        async move { register_user(input).await }
    });
```

### 9.4.4 — Create Verify Account Action

The original JS calls `verifyUser2()` immediately after successful registration. We'll do the same:

```rust
    // Create the server action for account verification (called after successful registration)
    let verify_action = Action::new(move |userid: &String| {
        let userid = userid.clone();
        async move { verify_account(userid).await }
    });
```

### 9.4.5 — Form Validity Memo

Create a derived signal that combines all validation states:

```rust
    // Compute overall form validity (client-side only)
    let form_valid = Memo::new(move |_| {
        let email_valid = is_valid_email(&email.get());
        let username_valid = is_valid_username(&username.get());
        let password_valid = is_valid_password(&password.get());
        let confirm_valid = password.get() == confirm_password.get() && !confirm_password.get().is_empty();
        let checkboxes_valid = privacy_checked.get() && eula_checked.get();
        
        email_valid && username_valid && password_valid && confirm_valid && checkboxes_valid
    });
```

### 9.4.6 — Submission Handler

Create a handler function that validates and dispatches:

```rust
    // Handler for registration form submission
    let on_register_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        
        // Clear any previous backend errors
        email_backend_error.set(None);
        username_backend_error.set(None);
        
        // Final client-side validation gate
        if !form_valid.get() {
            toast.show("Please correct the errors in the form", ToastType::Error);
            // Focus first invalid field (accessibility)
            focus_first_invalid_field();
            return;
        }
        
        // Build the registration input
        let input = RegistrationInput {
            email: email.get(),
            password: password.get(),
            username: username.get(),
            pkey: None,
            referral_uuid: referral_code.get(),
        };
        
        // Dispatch the action
        register_action.dispatch(input);
    };
```

### 9.4.7 — Handle Registration Response

Create an effect that watches the action result:

```rust
    // Handle register_action results reactively
    Effect::new(move |_| {
        if let Some(result) = register_action.value().get() {
            match result {
                Ok(response) => handle_registration_response(response),
                Err(e) => {
                    // Network error or server function error
                    log::error!("Registration error: {:?}", e);
                    toast.show(
                        "Connection error. Please check your network and try again.",
                        ToastType::Error,
                    );
                }
            }
        }
    });
    
    // Helper function to process the registration response
    let handle_registration_response = move |response: RegisterResponse| {
        match response.response_code.as_deref() {
            Some("10601") => {
                // Success! Call verify_account and proceed
                if let Some(userid) = &response.userid {
                    verify_action.dispatch(userid.clone());
                }
                
                // Store email in session storage for login page auto-fill
                if let Some(window) = web_sys::window() {
                    if let Ok(Some(storage)) = window.session_storage() {
                        let _ = storage.set_item("newUserEmail", &email.get());
                    }
                }
                
                // Show success toast
                toast.show(
                    user_friendly_msg("10601"),
                    ToastType::Success,
                );
                
                // Advance to step 3 (success screen)
                current_step.set(3);
            }
            
            Some("30601") => {
                // Duplicate email - show error on the email field
                email_backend_error.set(Some(
                    user_friendly_msg("30601").to_string()
                ));
                // Focus the email field for accessibility
                focus_field("email");
            }
            
            Some("30202") => {
                // Invalid username format - show error on username field
                username_backend_error.set(Some(
                    user_friendly_msg("30202").to_string()
                ));
                focus_field("username");
            }
            
            Some(code) => {
                // Other error - show toast
                toast.show(
                    user_friendly_msg(code),
                    ToastType::Error,
                );
            }
            
            None => {
                // No response code - unexpected
                toast.show(
                    "An unexpected error occurred. Please try again.",
                    ToastType::Error,
                );
            }
        }
    };
```

### 9.4.8 — Handle Verify Account Response (Optional Logging)

The `verify_account` call is fire-and-forget in the original JS, but we can log failures:

```rust
    // Handle verify_action results (mostly logging, doesn't affect UI flow)
    Effect::new(move |_| {
        if let Some(result) = verify_action.value().get() {
            match result {
                Ok(response) => {
                    log::info!("Account verification: {:?}", response.response_code);
                }
                Err(e) => {
                    // Log but don't interrupt user - they can verify later via email
                    log::warn!("Account verification failed: {:?}", e);
                }
            }
        }
    });
```

---

## 9.5 — UI Updates

### 9.5.1 — Register Button with Loading State

Update the Register button to show loading state and disabled state:

```rust
<button
    type="submit"
    id="registerBtn"
    class="btn btn-primary"
    disabled=move || register_action.pending().get() || !form_valid.get()
    aria-busy=move || register_action.pending().get()
>
    {move || {
        if register_action.pending().get() {
            view! {
                <span class="spinner" aria-hidden="true"></span>
                " Creating account..."
            }.into_any()
        } else {
            view! { "Create Account" }.into_any()
        }
    }}
</button>
```

### 9.5.2 — Email Field with Backend Error

The email field needs to display both client validation errors and backend errors:

```rust
<div
    class="input-field"
    class:invalid=move || !is_valid_email(&email.get()) || email_backend_error.get().is_some()
>
    <label for="email">Email</label>
    <input
        type="email"
        id="email"
        name="email"
        required
        aria-describedby="emailValidation"
        aria-invalid=move || !is_valid_email(&email.get()) || email_backend_error.get().is_some()
        prop:value=move || email.get()
        on:input=move |ev| {
            email.set(event_target_value(&ev));
            // Clear backend error when user starts typing
            email_backend_error.set(None);
        }
    />
    <span id="emailValidation" class="validation-message" role="alert">
        {move || {
            // Backend error takes precedence
            if let Some(err) = email_backend_error.get() {
                err
            } else if email.get().is_empty() {
                String::new()
            } else if !is_valid_email(&email.get()) {
                "Please enter a valid email address".to_string()
            } else {
                String::new()
            }
        }}
    </span>
</div>
```

### 9.5.3 — Username Field with Backend Error

Similar pattern for username:

```rust
<div
    class="input-field"
    class:invalid=move || !is_valid_username(&username.get()) || username_backend_error.get().is_some()
>
    <label for="username">Username</label>
    <input
        type="text"
        id="username"
        name="username"
        required
        minlength="3"
        maxlength="23"
        aria-describedby="usernameValidation"
        aria-invalid=move || !is_valid_username(&username.get()) || username_backend_error.get().is_some()
        prop:value=move || username.get()
        on:input=move |ev| {
            username.set(event_target_value(&ev));
            // Clear backend error when user starts typing
            username_backend_error.set(None);
        }
    />
    <span id="usernameValidation" class="validation-message" role="alert">
        {move || {
            if let Some(err) = username_backend_error.get() {
                err
            } else if username.get().is_empty() {
                String::new()
            } else if !is_valid_username(&username.get()) {
                "3-23 characters: letters, numbers, underscores, or hyphens".to_string()
            } else {
                String::new()
            }
        }}
    </span>
</div>
```

---

## 9.6 — Focus Management Helpers

Add utility functions for accessibility:

```rust
/// Focus a specific field by ID
fn focus_field(field_id: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(element) = document.get_element_by_id(field_id) {
                if let Some(input) = element.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = input.focus();
                }
            }
        }
    }
}

/// Focus the first invalid field in the registration form
fn focus_first_invalid_field() {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            // Query for first input with aria-invalid="true"
            if let Ok(Some(element)) = document.query_selector("[aria-invalid='true']") {
                if let Some(input) = element.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = input.focus();
                }
            }
        }
    }
}
```

---

## 9.7 — Screen Reader Announcements

Add step announcer for accessibility (from Step 8):

```rust
    // Announce form submission state changes
    Effect::new(move |_| {
        let announcer = get_or_create_announcer();
        
        if register_action.pending().get() {
            announcer.set_text_content(Some("Creating your account, please wait..."));
        }
    });
    
    // Announce registration result
    Effect::new(move |_| {
        if let Some(result) = register_action.value().get() {
            let announcer = get_or_create_announcer();
            match result {
                Ok(response) if response.response_code.as_deref() == Some("10601") => {
                    announcer.set_text_content(Some("Registration successful! Welcome to peer!"));
                }
                Ok(_) | Err(_) => {
                    announcer.set_text_content(Some("Registration error. Please check the form and try again."));
                }
            }
        }
    });
```

---

## 9.8 — Complete Code: Updated `src/pages/register.rs` (Step 2 Section)

Here's how the registration step (step 2) of the component should look after this implementation:

```rust
// Step 2: Registration Form
<div
    id="registrationStep"
    class="form-step"
    class:active=move || current_step.get() == 2
    aria-hidden=move || current_step.get() != 2
>
    <h2>"Create Your Account"</h2>
    
    <form id="registrationForm" on:submit=on_register_submit>
        // Email field (see 9.5.2)
        // Username field (see 9.5.3)
        // Password field (from Step 8)
        // Confirm password field (from Step 8)
        // Password strength meter (from Step 8)
        // Checkbox fields (from Step 8)
        
        <button
            type="submit"
            id="registerBtn"
            class="btn btn-primary"
            disabled=move || register_action.pending().get() || !form_valid.get()
            aria-busy=move || register_action.pending().get()
        >
            {move || {
                if register_action.pending().get() {
                    view! {
                        <span class="spinner" aria-hidden="true"></span>
                        " Creating account..."
                    }.into_any()
                } else {
                    view! { "Create Account" }.into_any()
                }
            }}
        </button>
    </form>
</div>
```

---

## 9.9 — Integration Test

Create an integration test for the registration round-trip:

### File: `tests/registration_submission.rs`

```rust
//! Integration tests for the registration submission flow.
//!
//! These tests require the mock backend to be running on port 4000.

use peer_web::api::registration::register_user;
use peer_web::models::user::RegistrationInput;

/// Test successful registration against mock backend
#[tokio::test]
async fn test_registration_success() {
    // Use a unique email to avoid duplicate errors
    let input = RegistrationInput {
        email: format!("test_{}@example.com", uuid::Uuid::new_v4()),
        password: "SecurePassword123".to_string(),
        username: format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]),
        pkey: None,
        referral_uuid: "85d5f836-b1f5-4c4e-9381-1b058e13df93".to_string(),
    };
    
    let result = register_user(input).await;
    
    assert!(result.is_ok(), "Registration should succeed");
    let response = result.unwrap();
    assert_eq!(response.status, "success");
    assert_eq!(response.response_code, Some("10601".to_string()));
    assert!(response.userid.is_some(), "Should return a user ID");
}

/// Test duplicate email returns correct error code
#[tokio::test]
async fn test_registration_duplicate_email() {
    let email = "existing@example.com"; // Mock backend should reject this
    
    let input = RegistrationInput {
        email: email.to_string(),
        password: "SecurePassword123".to_string(),
        username: "newuser123".to_string(),
        pkey: None,
        referral_uuid: "85d5f836-b1f5-4c4e-9381-1b058e13df93".to_string(),
    };
    
    let result = register_user(input).await;
    
    assert!(result.is_ok(), "Should return a response, not network error");
    let response = result.unwrap();
    assert_eq!(response.status, "error");
    assert_eq!(response.response_code, Some("30601".to_string()));
}

/// Test invalid referral UUID returns correct error code
#[tokio::test]
async fn test_registration_invalid_referral() {
    let input = RegistrationInput {
        email: "newuser@example.com".to_string(),
        password: "SecurePassword123".to_string(),
        username: "newuser123".to_string(),
        pkey: None,
        referral_uuid: "00000000-0000-0000-0000-000000000000".to_string(), // Non-existent
    };
    
    let result = register_user(input).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.response_code, Some("31007".to_string()));
}

/// Test missing required fields returns correct error code
#[tokio::test]
async fn test_registration_missing_fields() {
    let input = RegistrationInput {
        email: "".to_string(), // Empty email
        password: "SecurePassword123".to_string(),
        username: "newuser123".to_string(),
        pkey: None,
        referral_uuid: "85d5f836-b1f5-4c4e-9381-1b058e13df93".to_string(),
    };
    
    let result = register_user(input).await;
    
    // Server-side validation should catch this before GraphQL
    assert!(result.is_err() || {
        let response = result.unwrap();
        response.response_code == Some("30301".to_string())
    });
}
```

---

## 9.10 — Mock Backend Expectations

Update the mock backend to handle registration scenarios:

### File: `packages/mock_backend/resolvers/register.js`

```javascript
const KNOWN_EMAILS = new Set(['existing@example.com', 'taken@peer.com']);
const KNOWN_REFERRALS = new Set(['85d5f836-b1f5-4c4e-9381-1b058e13df93']);

module.exports = {
  Mutation: {
    register: (_, { input }) => {
      const { email, password, username, referralUuid } = input;
      
      // Check for missing required fields
      if (!email || !password || !username) {
        return {
          status: 'error',
          ResponseCode: '30301',
          userid: null,
        };
      }
      
      // Check for duplicate email
      if (KNOWN_EMAILS.has(email.toLowerCase())) {
        return {
          status: 'error',
          ResponseCode: '30601',
          userid: null,
        };
      }
      
      // Check for invalid referral
      if (referralUuid && !KNOWN_REFERRALS.has(referralUuid)) {
        return {
          status: 'error',
          ResponseCode: '31007',
          userid: null,
        };
      }
      
      // Validate username format
      if (!/^[a-zA-Z0-9_-]{3,23}$/.test(username)) {
        return {
          status: 'error',
          ResponseCode: '30202',
          userid: null,
        };
      }
      
      // Success - generate a mock user ID
      const userid = require('crypto').randomUUID();
      
      // Remember this email as taken for future requests
      KNOWN_EMAILS.add(email.toLowerCase());
      
      return {
        status: 'success',
        ResponseCode: '10601',
        userid,
      };
    },
    
    verifyAccount: (_, { userid }) => {
      // Always succeed for mock
      return {
        status: 'success',
        ResponseCode: '10701',
      };
    },
  },
};
```

---

## 9.11 — Testing Checklist

### Manual Testing

| # | Test Case | Steps | Expected Result |
|---|-----------|-------|-----------------|
| 1 | Happy path | Enter valid referral → fill all fields correctly → submit | Step 3 visible, success toast, email in sessionStorage |
| 2 | Duplicate email | Use `existing@example.com` | Email field shows "Email already registered", field is focused |
| 3 | Invalid username | Use `ab` (too short) | Submit button disabled, username validation shown |
| 4 | Password mismatch | Enter different password & confirm | Submit button disabled, "Passwords do not match" shown |
| 5 | Unchecked checkboxes | Leave one checkbox unchecked | Submit button disabled, checkbox error shown |
| 6 | Loading state | Submit valid form | Button shows spinner, is disabled during request |
| 7 | Network error | Stop mock backend, submit | Error toast: "Connection error..." |
| 8 | Clearing backend error | Get duplicate email error → type new email | Backend error clears as user types |
| 9 | Screen reader | Use VoiceOver | Submission state announced, errors announced |
| 10 | Keyboard only | Navigate & submit with Tab/Enter | Full flow completable without mouse |

### Automated Testing

```bash
# Run integration tests (mock backend must be running)
cargo test --features ssr --test registration_submission

# Run all server function tests
cargo test --features ssr --test server_functions
```

---

## 9.12 — Files Modified/Created

| File | Action | Description |
|------|--------|-------------|
| `src/pages/register.rs` | Modified | Added register action, effects, backend error signals, submit handler |
| `src/utils/response_codes.rs` | Modified | Added registration-specific response code mappings |
| `tests/registration_submission.rs` | Created | Integration tests for registration flow |
| `packages/mock_backend/resolvers/register.js` | Modified | Added register mutation resolver |

---

## 9.13 — Troubleshooting

### "Button stays disabled even with valid form"

Check that all validation memos are correctly computing. Use browser devtools to inspect signal values:

```rust
Effect::new(move |_| {
    log::debug!("Form validity: {}", form_valid.get());
    log::debug!("Email valid: {}", is_valid_email(&email.get()));
    log::debug!("Username valid: {}", is_valid_username(&username.get()));
    // ... etc
});
```

### "Action never completes"

1. Verify mock backend is running: `curl http://localhost:4000/graphql`
2. Check browser Network tab for the `/api/register_user` request
3. Look for panics in the Axum server console

### "Backend error not clearing"

Ensure the `on:input` handler sets `email_backend_error.set(None)`:

```rust
on:input=move |ev| {
    email.set(event_target_value(&ev));
    email_backend_error.set(None); // Must be present!
}
```

### "sessionStorage not setting"

1. Check browser console for errors (CORS, permissions)
2. Verify `web_sys::window()` is available (not during SSR)
3. Wrap in `create_effect` to ensure it runs on client:

```rust
Effect::new(move |_| {
    // This runs only on the client
    if let Some(window) = web_sys::window() {
        // ...
    }
});
```

---

## 9.14 — Next Steps

After completing this step:

1. **Step 10**: Build the success confirmation screen (step 3)
2. **Step 11**: Implement navigation and back button behaviour
3. **Step 12**: Create the reusable toast component (if not already done)

The registration flow's core data path is now complete. Users can enter a referral code, fill out the form, submit, and receive appropriate feedback for all success and error scenarios.
