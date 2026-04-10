# Step 8 — Step 2 UI: Registration Form Fields

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Build the registration form (Step 2 of the flow) with email, username, password, confirm-password, and checkbox fields — all with real-time client-side validation. This step produces the reactive UI only; the server round-trip (actually calling `register_user`) is wired in Step 9.

**Depends on:** Step 5 (Router & Page Shell)

---

## 8.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 5 complete | Navigate to `http://localhost:3000/register` | Renders the page shell with `.container` div |
| Page component exists | Check `src/pages/register.rs` | Contains `RegisterPage` component with step management |
| Router configured | Check `src/app.rs` | Has `<Route path="/register" view=RegisterPage />` |
| CSS imported | Check `style/main.scss` or `public/` | `login-register.css` is loadable |
| Shared types | Check `src/models/user.rs` | `RegistrationInput`, `RegisterResponse` types defined |
| Validation module | Check `src/components/validation.rs` | `is_valid_uuid` function exists (from Step 6) |
| Leptos dev server runs | `cargo leptos watch` | Compiles and serves at `localhost:3000` |

---

## 8.2 — Architecture Overview

Step 8 is purely client-side UI. No server functions are called yet — that happens in Step 9. The goal is to render the registration form HTML, validate all inputs reactively, and display the password strength meter.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  RegisterPage Component (src/pages/register.rs)                              │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  Signals (owned by parent):                                            │  │
│  │    current_step: RwSignal<RegistrationStep>                            │  │
│  │    referral_code: RwSignal<String>           ← from Step 6             │  │
│  │    email: RwSignal<String>                                             │  │
│  │    username: RwSignal<String>                                          │  │
│  │    password: RwSignal<String>                                          │  │
│  │    confirm_password: RwSignal<String>                                  │  │
│  │    privacy_accepted: RwSignal<bool>                                    │  │
│  │    eula_accepted: RwSignal<bool>                                       │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  RegistrationStep Component (src/components/registration_form.rs)      │  │
│  │    ├── EmailField (input + validation icon + message)                  │  │
│  │    ├── UsernameField (input + validation icon + message)               │  │
│  │    ├── PasswordField (input + toggle + strength meter)                 │  │
│  │    ├── ConfirmPasswordField (input + toggle + message)                 │  │
│  │    ├── CheckboxGroup (Privacy Policy + EULA)                           │  │
│  │    └── Submit button ("Create Account")                                │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  Validation (src/components/validation.rs)                             │  │
│  │    ├── is_valid_email(input) → bool                                    │  │
│  │    ├── is_valid_username(input) → bool                                 │  │
│  │    ├── validate_password(input) → PasswordValidation                   │  │
│  │    ├── passwords_match(password, confirm) → bool                       │  │
│  │    └── PasswordStrength enum (VeryWeak..Excellent)                     │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │  PasswordStrengthMeter Component (src/components/password_strength.rs) │  │
│  │    ├── 5-segment visual meter                                          │  │
│  │    ├── Strength label (Very Weak → Excellent)                          │  │
│  │    └── Unmet requirements list                                         │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Separate `RegistrationStep` component | Keeps `RegisterPage` clean; mirrors `ReferralStep` pattern from Step 6 |
| Client-side validation only (no server call) | Step 9 adds the server round-trip; separation makes both steps independently testable |
| Signals owned by parent | Parent owns the signals so data persists when navigating back from step 3 (Step 11) |
| Derived signals for validity | `is_email_valid`, `is_password_strong`, etc. are computed — single source of truth |
| Password requirements as struct | `PasswordValidation` struct holds individual checks; enables requirement list rendering |
| Checkbox signals are booleans | Simpler than string signals; directly map to `checked` attribute |

---

## 8.3 — Files to Create / Modify

| File | Action | Purpose |
|------|--------|---------|
| `src/components/mod.rs` | Modify | Add `pub mod registration_form;` and `pub mod password_strength;` |
| `src/components/registration_form.rs` | **Create** | `RegistrationStep` component with all form fields |
| `src/components/password_strength.rs` | **Create** | `PasswordStrengthMeter` component |
| `src/components/validation.rs` | Modify | Add email, username, password validation functions |
| `src/pages/register.rs` | Modify | Add registration form signals and render `RegistrationStep` |

---

## 8.4 — Client-Side Validation Functions

### Extend `src/components/validation.rs`

Add the following validation functions alongside the existing `is_valid_uuid`:

#### 8.4.1 — Email Validation

```rust
/// Validate email format.
///
/// Regex: `/^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/`
///
/// # Examples
/// ```
/// assert!(is_valid_email("user@example.com"));
/// assert!(is_valid_email("user.name+tag@sub.domain.org"));
/// assert!(!is_valid_email("user@"));
/// assert!(!is_valid_email("@example.com"));
/// assert!(!is_valid_email("plaintext"));
/// ```
pub fn is_valid_email(input: &str) -> bool {
    let input = input.trim();
    
    // Quick length check
    if input.len() < 5 || input.len() > 254 {
        return false;
    }
    
    // Must contain exactly one @
    let at_count = input.chars().filter(|&c| c == '@').count();
    if at_count != 1 {
        return false;
    }
    
    let parts: Vec<&str> = input.split('@').collect();
    let (local, domain) = (parts[0], parts[1]);
    
    // Local part: non-empty, alphanumeric + ._%+-
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    if !local.chars().all(|c| {
        c.is_ascii_alphanumeric() || "._%+-".contains(c)
    }) {
        return false;
    }
    
    // Domain part: must have at least one dot, valid chars
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }
    
    let domain_parts: Vec<&str> = domain.split('.').collect();
    
    // TLD must be at least 2 chars
    if let Some(tld) = domain_parts.last() {
        if tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }
    }
    
    // All domain parts must be valid
    domain_parts.iter().all(|part| {
        !part.is_empty() && part.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-'
        })
    })
}
```

> **Why not use the `regex` crate?** Same rationale as Step 6: saves ~200 KB in WASM binary. The character-by-character validation is zero-dependency.

#### 8.4.2 — Username Validation

```rust
/// Validate username format.
///
/// Rules:
/// - 3–23 characters
/// - Alphanumeric, underscore, or hyphen only
/// - Regex equivalent: `/^[a-zA-Z0-9_-]{3,23}$/`
///
/// # Examples
/// ```
/// assert!(is_valid_username("peer_user"));
/// assert!(is_valid_username("user-123"));
/// assert!(is_valid_username("abc"));       // 3 chars minimum
/// assert!(!is_valid_username("ab"));       // too short
/// assert!(!is_valid_username("user@name")); // invalid char
/// assert!(!is_valid_username("this_username_is_way_too_long_for_the_system")); // >23 chars
/// ```
pub fn is_valid_username(input: &str) -> bool {
    let input = input.trim();
    let len = input.len();
    
    if len < 3 || len > 23 {
        return false;
    }
    
    input.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    })
}
```

#### 8.4.3 — Password Validation

Password validation is more complex — it needs to return both overall validity and individual requirement states for the UI.

```rust
/// Individual password requirement checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordRequirements {
    /// At least 8 characters
    pub length: bool,
    /// Contains at least one lowercase letter
    pub lowercase: bool,
    /// Contains at least one uppercase letter
    pub uppercase: bool,
    /// Contains at least one digit
    pub number: bool,
    /// Contains a special character OR is 12+ characters
    pub special: bool,
}

impl PasswordRequirements {
    /// Check if all required criteria are met.
    /// Required: length, lowercase, uppercase, number
    /// Optional (bonus): special
    pub fn is_sufficient(&self) -> bool {
        self.length && self.lowercase && self.uppercase && self.number
    }
    
    /// Count how many requirements are met (out of 5).
    pub fn met_count(&self) -> u8 {
        let mut count = 0;
        if self.length { count += 1; }
        if self.lowercase { count += 1; }
        if self.uppercase { count += 1; }
        if self.number { count += 1; }
        if self.special { count += 1; }
        count
    }
}

/// Password strength levels for the meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// 0–1 requirements met
    VeryWeak,
    /// 2 requirements met
    Weak,
    /// 3 requirements met
    NeedsImprovement,
    /// All 4 required criteria met
    Good,
    /// All 4 required + special character
    Excellent,
}

impl PasswordStrength {
    /// CSS class suffix for the strength meter.
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::VeryWeak => "weak",
            Self::Weak => "weak2",
            Self::NeedsImprovement => "medium",
            Self::Good => "strong",
            Self::Excellent => "excellent",
        }
    }
    
    /// Human-readable label for screen readers and display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::VeryWeak => "Very weak",
            Self::Weak => "Weak",
            Self::NeedsImprovement => "Needs improvement",
            Self::Good => "Good",
            Self::Excellent => "Excellent",
        }
    }
    
    /// CSS class for the label text.
    pub fn label_class(&self) -> &'static str {
        match self {
            Self::VeryWeak => "very-weak",
            Self::Weak => "weak",
            Self::NeedsImprovement => "improvement",
            Self::Good => "good",
            Self::Excellent => "excellent",
        }
    }
}

/// Validate a password and return detailed requirements + strength.
///
/// # Examples
/// ```
/// let result = validate_password("Abcd1234");
/// assert!(result.requirements.is_sufficient());
/// assert_eq!(result.strength, PasswordStrength::Good);
///
/// let result = validate_password("Abcd1234!");
/// assert_eq!(result.strength, PasswordStrength::Excellent);
///
/// let result = validate_password("abcd");
/// assert!(!result.requirements.is_sufficient());
/// assert_eq!(result.strength, PasswordStrength::VeryWeak);
/// ```
pub struct PasswordValidation {
    pub requirements: PasswordRequirements,
    pub strength: PasswordStrength,
}

pub fn validate_password(password: &str) -> PasswordValidation {
    let requirements = PasswordRequirements {
        length: password.len() >= 8,
        lowercase: password.chars().any(|c| c.is_ascii_lowercase()),
        uppercase: password.chars().any(|c| c.is_ascii_uppercase()),
        number: password.chars().any(|c| c.is_ascii_digit()),
        special: password.chars().any(|c| {
            "!@#$%^&*(),.?\":{}|<>".contains(c)
        }) || password.len() >= 12,
    };
    
    let strength = if requirements.is_sufficient() {
        if requirements.special {
            PasswordStrength::Excellent
        } else {
            PasswordStrength::Good
        }
    } else {
        match requirements.met_count() {
            0 | 1 => PasswordStrength::VeryWeak,
            2 => PasswordStrength::Weak,
            _ => PasswordStrength::NeedsImprovement,
        }
    };
    
    PasswordValidation { requirements, strength }
}
```

#### 8.4.4 — Confirm Password Validation

```rust
/// Check if confirm password matches the original password.
///
/// Both values are trimmed before comparison.
pub fn passwords_match(password: &str, confirm_password: &str) -> bool {
    !confirm_password.is_empty() && password == confirm_password
}
```

### Unit Tests (add to `src/components/validation.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Existing UUID tests from Step 6...

    // --- Email tests ---
    
    #[test]
    fn valid_email_simple() {
        assert!(is_valid_email("user@example.com"));
    }
    
    #[test]
    fn valid_email_with_plus() {
        assert!(is_valid_email("user+tag@example.com"));
    }
    
    #[test]
    fn valid_email_subdomain() {
        assert!(is_valid_email("user@mail.sub.example.org"));
    }
    
    #[test]
    fn invalid_email_no_at() {
        assert!(!is_valid_email("userexample.com"));
    }
    
    #[test]
    fn invalid_email_no_domain() {
        assert!(!is_valid_email("user@"));
    }
    
    #[test]
    fn invalid_email_no_tld() {
        assert!(!is_valid_email("user@example"));
    }
    
    #[test]
    fn invalid_email_short_tld() {
        assert!(!is_valid_email("user@example.x"));
    }
    
    // --- Username tests ---
    
    #[test]
    fn valid_username_min_length() {
        assert!(is_valid_username("abc"));
    }
    
    #[test]
    fn valid_username_max_length() {
        assert!(is_valid_username("abcdefghij12345678901ab")); // 23 chars
    }
    
    #[test]
    fn valid_username_with_underscore() {
        assert!(is_valid_username("peer_user"));
    }
    
    #[test]
    fn valid_username_with_hyphen() {
        assert!(is_valid_username("peer-user"));
    }
    
    #[test]
    fn invalid_username_too_short() {
        assert!(!is_valid_username("ab"));
    }
    
    #[test]
    fn invalid_username_too_long() {
        assert!(!is_valid_username("abcdefghij123456789012ab")); // 24 chars
    }
    
    #[test]
    fn invalid_username_special_chars() {
        assert!(!is_valid_username("user@name"));
        assert!(!is_valid_username("user.name"));
        assert!(!is_valid_username("user name"));
    }
    
    // --- Password tests ---
    
    #[test]
    fn password_very_weak() {
        let result = validate_password("abc");
        assert_eq!(result.strength, PasswordStrength::VeryWeak);
        assert!(!result.requirements.is_sufficient());
    }
    
    #[test]
    fn password_weak() {
        let result = validate_password("abcdefgh");
        assert_eq!(result.strength, PasswordStrength::Weak);
    }
    
    #[test]
    fn password_needs_improvement() {
        let result = validate_password("Abcdefgh");
        assert_eq!(result.strength, PasswordStrength::NeedsImprovement);
    }
    
    #[test]
    fn password_good() {
        let result = validate_password("Abcd1234");
        assert_eq!(result.strength, PasswordStrength::Good);
        assert!(result.requirements.is_sufficient());
    }
    
    #[test]
    fn password_excellent_with_special() {
        let result = validate_password("Abcd1234!");
        assert_eq!(result.strength, PasswordStrength::Excellent);
    }
    
    #[test]
    fn password_excellent_with_length() {
        let result = validate_password("Abcdefgh1234"); // 12 chars, no special
        assert_eq!(result.strength, PasswordStrength::Excellent);
    }
    
    // --- Confirm password tests ---
    
    #[test]
    fn passwords_match_valid() {
        assert!(passwords_match("Abcd1234", "Abcd1234"));
    }
    
    #[test]
    fn passwords_match_empty_confirm() {
        assert!(!passwords_match("Abcd1234", ""));
    }
    
    #[test]
    fn passwords_match_different() {
        assert!(!passwords_match("Abcd1234", "Abcd12345"));
    }
}
```

---

## 8.5 — Password Strength Meter Component

### Create `src/components/password_strength.rs`

This component renders the visual password strength indicator matching the existing PHP/JS implementation.

```rust
use leptos::prelude::*;
use crate::components::validation::{PasswordValidation, PasswordStrength, PasswordRequirements};

/// Props for the password strength meter.
#[component]
pub fn PasswordStrengthMeter(
    /// The current password validation state — updates reactively.
    validation: Memo<PasswordValidation>,
    /// Whether to show the meter (hidden when password is empty).
    visible: Memo<bool>,
) -> impl IntoView {
    // Derived: CSS class for the strength fill bar
    let fill_class = Memo::new(move |_| {
        format!("strength-fill {}", validation.get().strength.css_class())
    });
    
    // Derived: which strength label to show
    let strength_label = Memo::new(move |_| {
        validation.get().strength.label()
    });
    
    let label_class = Memo::new(move |_| {
        validation.get().strength.label_class()
    });
    
    // Derived: which requirements are still unmet
    let requirements = Memo::new(move |_| {
        validation.get().requirements
    });

    view! {
        <div
            class=move || {
                if visible.get() {
                    "password-strength show"
                } else {
                    "password-strength none"
                }
            }
            id="passwordStrength"
        >
            // Strength labels — only show current strength
            <div class="strength-labels medium_font">
                <Show when=move || label_class.get() == "very-weak">
                    <span class="strength-text very-weak">{strength_label.get()}</span>
                </Show>
                <Show when=move || label_class.get() == "weak">
                    <span class="strength-text weak">{strength_label.get()}</span>
                </Show>
                <Show when=move || label_class.get() == "improvement">
                    <span class="strength-text improvement">{strength_label.get()}</span>
                </Show>
                <Show when=move || label_class.get() == "good">
                    <span class="strength-text good">{strength_label.get()}</span>
                </Show>
                <Show when=move || label_class.get() == "excellent">
                    <span class="strength-text excellent">{strength_label.get()}</span>
                </Show>
            </div>

            // Visual strength meter bar
            <div class="strength-meter">
                <div class=move || fill_class.get() id="strengthFill">
                    <span class="strength-segment segment-weak"></span>
                    <span class="strength-segment segment-weak2"></span>
                    <span class="strength-segment segment-medium"></span>
                    <span class="strength-segment segment-strong"></span>
                    <span class="strength-segment segment-excellent"></span>
                </div>
            </div>

            // Unmet requirements list
            <ul
                class="strength-requirements medium_font"
                role="list"
                aria-label="Password requirements"
            >
                <li
                    id="lengthReq"
                    class=move || {
                        if requirements.get().length { "none" } else { "show" }
                    }
                >
                    "Min. 8 chars,"
                </li>
                <li
                    id="lowerReq"
                    class=move || {
                        if requirements.get().lowercase { "none" } else { "show" }
                    }
                >
                    "1 lowercase"
                </li>
                <li
                    id="upperReq"
                    class=move || {
                        if requirements.get().uppercase { "none" } else { "show" }
                    }
                >
                    "1 uppercase,"
                </li>
                <li
                    id="numberReq"
                    class=move || {
                        if requirements.get().number { "none" } else { "show" }
                    }
                >
                    "1 number,"
                </li>
            </ul>
        </div>
    }
}
```

---

## 8.6 — Registration Form Component

### Create `src/components/registration_form.rs`

This is the main deliverable of Step 8. The component renders the full registration form (Step 2 of the flow) with all input fields.

```rust
use leptos::prelude::*;
use crate::components::validation::{
    is_valid_email, is_valid_username, validate_password, passwords_match
};
use crate::components::password_strength::PasswordStrengthMeter;

/// Props for the RegistrationStep component.
///
/// The parent (`RegisterPage`) owns the signals so that:
/// - Form data persists when navigating back (Step 11)
/// - `on_submit` can be swapped between a no-op (Step 8) and the real
///   server call (Step 9)
#[component]
pub fn RegistrationStep(
    /// Email signal, owned by parent.
    email: RwSignal<String>,
    /// Username signal, owned by parent.
    username: RwSignal<String>,
    /// Password signal, owned by parent.
    password: RwSignal<String>,
    /// Confirm password signal, owned by parent.
    confirm_password: RwSignal<String>,
    /// Privacy policy checkbox state.
    privacy_accepted: RwSignal<bool>,
    /// EULA checkbox state.
    eula_accepted: RwSignal<bool>,
    /// Callback invoked when the form is submitted with valid data.
    /// In Step 8 this simply advances to step 3; in Step 9 it triggers
    /// the server function.
    on_submit: Action<(), ()>,
) -> impl IntoView {
    // ... component implementation
}
```

### 8.6.1 — Signals & Derived State

Inside the component body:

```rust
// --- Password visibility toggles ---
let show_password = RwSignal::new(false);
let show_confirm_password = RwSignal::new(false);

// --- Derived validation states ---

// Email validation
let is_email_valid = Memo::new(move |_| {
    is_valid_email(&email.get())
});
let email_message = Memo::new(move |_| {
    let e = email.get();
    if e.is_empty() {
        String::new()
    } else if is_email_valid.get() {
        String::new()
    } else {
        "Please enter a valid email address".to_string()
    }
});
let email_field_class = Memo::new(move |_| {
    let e = email.get();
    if e.is_empty() {
        "input-field"
    } else if is_email_valid.get() {
        "input-field valid"
    } else {
        "input-field invalid"
    }
});

// Username validation
let is_username_valid = Memo::new(move |_| {
    is_valid_username(&username.get())
});
let username_message = Memo::new(move |_| {
    let u = username.get();
    if u.is_empty() {
        String::new()
    } else if is_username_valid.get() {
        String::new()
    } else {
        "Username must be 3-23 characters".to_string()
    }
});
let username_field_class = Memo::new(move |_| {
    let u = username.get();
    if u.is_empty() {
        "input-field"
    } else if is_username_valid.get() {
        "input-field valid"
    } else {
        "input-field invalid"
    }
});

// Password validation
let password_validation = Memo::new(move |_| {
    validate_password(&password.get())
});
let is_password_valid = Memo::new(move |_| {
    password_validation.get().requirements.is_sufficient()
});
let password_visible = Memo::new(move |_| {
    !password.get().is_empty()
});
let password_field_class = Memo::new(move |_| {
    let p = password.get();
    if p.is_empty() {
        "input-field"
    } else if is_password_valid.get() {
        "input-field valid"
    } else {
        "input-field invalid"
    }
});

// Confirm password validation
let is_confirm_valid = Memo::new(move |_| {
    passwords_match(&password.get(), &confirm_password.get())
});
let confirm_message = Memo::new(move |_| {
    let c = confirm_password.get();
    if c.is_empty() {
        String::new()
    } else if is_confirm_valid.get() {
        String::new()
    } else {
        "Passwords do not match".to_string()
    }
});
let confirm_field_class = Memo::new(move |_| {
    let c = confirm_password.get();
    if c.is_empty() {
        "input-field"
    } else if is_confirm_valid.get() {
        "input-field valid"
    } else {
        "input-field invalid"
    }
});

// Checkbox validation
let checkbox_message = RwSignal::new(String::new());
let checkbox_error_shown = Memo::new(move |_| {
    !checkbox_message.get().is_empty()
});

// Overall form validity (for submit button)
let is_form_valid = Memo::new(move |_| {
    is_email_valid.get()
        && is_username_valid.get()
        && is_password_valid.get()
        && is_confirm_valid.get()
        && privacy_accepted.get()
        && eula_accepted.get()
});
```

### 8.6.2 — Form Submission Handler

```rust
// Handle form submission
let handle_submit = move |ev: web_sys::SubmitEvent| {
    ev.prevent_default();
    
    // Validate checkboxes (set message if not checked)
    if !privacy_accepted.get() && !eula_accepted.get() {
        checkbox_message.set(
            "Please accept both the Privacy Policy and EULA".to_string()
        );
        return;
    } else if !privacy_accepted.get() {
        checkbox_message.set(
            "Please accept the Privacy Policy to continue".to_string()
        );
        return;
    } else if !eula_accepted.get() {
        checkbox_message.set(
            "Please accept the End User License Agreement (EULA) to continue".to_string()
        );
        return;
    }
    
    // Clear checkbox message if both are checked
    checkbox_message.set(String::new());
    
    // Validate all fields
    if !is_form_valid.get() {
        // Focus first invalid field (Step 13 will enhance this)
        return;
    }
    
    // Dispatch the submit action
    on_submit.dispatch(());
};
```

### 8.6.3 — View Markup

The `view!` macro output should mirror the existing `register.php` DOM structure.

```rust
view! {
    <div class="form-step" id="registrationStep" data-step="2">
        <div class="step-header">
            <h2 class="x_large_font">"Register"</h2>
            <p class="large_font">
                "Create your account in few seconds and start earning on your favorite content."
            </p>
        </div>

        <form id="registrationForm" novalidate=true on:submit=handle_submit>
            // --- Email Field ---
            <div class="input-group">
                <div class=move || email_field_class.get() id="emailField">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-envelope"></i>
                    </span>
                    <input
                        type="email"
                        id="email"
                        name="email"
                        placeholder="Enter your email"
                        required=true
                        aria-describedby="emailValidation emailHelp"
                        autocomplete="email"
                        prop:value=move || email.get()
                        on:input=move |ev| {
                            email.set(event_target_value(&ev));
                        }
                    />
                    <span
                        class=move || {
                            if is_email_valid.get() && !email.get().is_empty() {
                                "validation-icon show"
                            } else {
                                "validation-icon"
                            }
                        }
                        id="emailValidIcon"
                        aria-hidden="true"
                    >
                        <i class="peer-icon peer-icon-tick-circle"></i>
                    </span>
                </div>
                <div
                    class="validation-message medium_font"
                    id="emailValidation"
                    role="alert"
                    aria-live="polite"
                >
                    {move || email_message.get()}
                </div>
                <div id="emailHelp" class="sr-only">
                    "Enter a valid email address"
                </div>
            </div>

            // --- Username Field ---
            <div class="input-group">
                <div class=move || username_field_class.get() id="usernameField">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-user"></i>
                    </span>
                    <input
                        type="text"
                        id="username"
                        name="username"
                        placeholder="Choose a username"
                        required=true
                        aria-describedby="usernameValidation usernameHelp"
                        autocomplete="username"
                        prop:value=move || username.get()
                        on:input=move |ev| {
                            username.set(event_target_value(&ev));
                        }
                    />
                    <span
                        class=move || {
                            if is_username_valid.get() && !username.get().is_empty() {
                                "validation-icon show"
                            } else {
                                "validation-icon"
                            }
                        }
                        id="usernameValidIcon"
                        aria-hidden="true"
                    >
                        <i class="peer-icon peer-icon-tick-circle"></i>
                    </span>
                </div>
                <div
                    class="validation-message medium_font"
                    id="usernameValidation"
                    role="alert"
                    aria-live="polite"
                >
                    {move || username_message.get()}
                </div>
                <div id="usernameHelp" class="sr-only">
                    "Username must be 3-23 characters"
                </div>
            </div>

            // --- Password Field ---
            <div class="input-group">
                <div class=move || password_field_class.get() id="passwordField">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-lock"></i>
                    </span>
                    <input
                        type=move || if show_password.get() { "text" } else { "password" }
                        id="password"
                        name="password"
                        placeholder="Create a strong password"
                        required=true
                        aria-describedby="passwordValidation passwordHelp passwordStrength"
                        autocomplete="new-password"
                        prop:value=move || password.get()
                        on:input=move |ev| {
                            password.set(event_target_value(&ev));
                        }
                    />
                    <span
                        class="toggle-passwordBtn-icon"
                        id="togglePasswordBtn"
                        title="Toggle password visibility"
                        aria-label=move || {
                            if show_password.get() { "Hide password" } else { "Show password" }
                        }
                        on:click=move |_| {
                            show_password.update(|v| *v = !*v);
                        }
                    >
                        <i class=move || {
                            if show_password.get() {
                                "peer-icon peer-icon-eye-open"
                            } else {
                                "peer-icon peer-icon-eye-close"
                            }
                        }></i>
                    </span>
                </div>
                <div
                    class="validation-message medium_font"
                    id="passwordValidation"
                    role="alert"
                    aria-live="polite"
                >
                </div>

                // Password strength meter component
                <PasswordStrengthMeter
                    validation=password_validation
                    visible=password_visible
                />
                
                <div id="passwordHelp" class="sr-only">
                    "Password must meet all security requirements"
                </div>
            </div>

            // --- Confirm Password Field ---
            <div class="input-group">
                <div class=move || confirm_field_class.get() id="confirmPasswordField">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-lock"></i>
                    </span>
                    <input
                        type=move || if show_confirm_password.get() { "text" } else { "password" }
                        id="confirmPassword"
                        name="confirmPassword"
                        placeholder="Confirm your password"
                        required=true
                        aria-describedby="confirmPasswordValidation confirmPasswordHelp"
                        autocomplete="new-password"
                        prop:value=move || confirm_password.get()
                        on:input=move |ev| {
                            confirm_password.set(event_target_value(&ev));
                        }
                    />
                    <span
                        class="toggle-passwordBtn-icon"
                        id="toggleConfirmPasswordBtn"
                        title="Toggle confirm password visibility"
                        aria-label=move || {
                            if show_confirm_password.get() { "Hide password" } else { "Show password" }
                        }
                        on:click=move |_| {
                            show_confirm_password.update(|v| *v = !*v);
                        }
                    >
                        <i class=move || {
                            if show_confirm_password.get() {
                                "peer-icon peer-icon-eye-open"
                            } else {
                                "peer-icon peer-icon-eye-close"
                            }
                        }></i>
                    </span>
                </div>
                <div
                    class="validation-message medium_font"
                    id="confirmPasswordValidation"
                    role="alert"
                    aria-live="polite"
                >
                    {move || confirm_message.get()}
                </div>
                <div id="confirmPasswordHelp" class="sr-only">
                    "Re-enter your password to confirm"
                </div>
            </div>

            // --- Checkbox Fields ---
            <div class="input-group">
                // Privacy Policy checkbox
                <div
                    class=move || {
                        if checkbox_error_shown.get() && !privacy_accepted.get() {
                            "checkbox-field checkbox-error"
                        } else {
                            "checkbox-field"
                        }
                    }
                    id="readPrivacyField"
                >
                    <label class="checkbox-wrapper" for="readPrivacy">
                        <input
                            type="checkbox"
                            id="readPrivacy"
                            name="readPrivacy"
                            aria-describedby="checkboxValidation"
                            prop:checked=move || privacy_accepted.get()
                            on:change=move |ev| {
                                privacy_accepted.set(event_target_checked(&ev));
                                // Clear error when checkbox changes
                                if privacy_accepted.get() && eula_accepted.get() {
                                    checkbox_message.set(String::new());
                                }
                            }
                        />
                        <span class="checkbox-label medium_font">
                            "I agree to the "
                            <a href="https://peerapp.de/privacy.html" target="_blank">
                                "Privacy Policy."
                            </a>
                        </span>
                    </label>
                </div>

                // EULA checkbox
                <div
                    class=move || {
                        if checkbox_error_shown.get() && !eula_accepted.get() {
                            "checkbox-field checkbox-error"
                        } else {
                            "checkbox-field"
                        }
                    }
                    id="agreementEULAField"
                >
                    <label class="checkbox-wrapper" for="agreementEULA">
                        <input
                            type="checkbox"
                            id="agreementEULA"
                            name="agreementEULA"
                            aria-describedby="checkboxValidation"
                            prop:checked=move || eula_accepted.get()
                            on:change=move |ev| {
                                eula_accepted.set(event_target_checked(&ev));
                                // Clear error when checkbox changes
                                if privacy_accepted.get() && eula_accepted.get() {
                                    checkbox_message.set(String::new());
                                }
                            }
                        />
                        <span class="checkbox-label medium_font">
                            "I agree to the "
                            <a href="https://peerapp.de/EULA.html" target="_blank">
                                "End User License Agreement (EULA)"
                            </a>
                            "."
                        </span>
                    </label>
                </div>

                // Checkbox validation message
                <div
                    class=move || {
                        if checkbox_error_shown.get() {
                            "validation-message medium_font notvalid"
                        } else {
                            "validation-message medium_font"
                        }
                    }
                    id="checkboxValidation"
                    role="alert"
                    aria-live="polite"
                >
                    {move || checkbox_message.get()}
                </div>
            </div>

            // --- Submit Button ---
            <button type="submit" class="btn btn-primary" id="registerBtn">
                "Create Account"
            </button>

            // --- Already Registered Link ---
            <div class="already_register medium_font">
                <p>
                    "Already registered? "
                    <a href="/login">"Login here"</a>
                </p>
            </div>
        </form>
    </div>
}
```

---

## 8.7 — RegisterPage Integration

### Update `src/pages/register.rs`

Add the registration form signals and render the `RegistrationStep` component when `current_step == RegistrationStep::Registration`.

```rust
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use crate::components::referral::ReferralStep;
use crate::components::registration_form::RegistrationStep;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationStep {
    Referral,
    Registration,
    Success,
}

impl Default for RegistrationStep {
    fn default() -> Self {
        Self::Referral
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    // Step state
    let current_step = RwSignal::new(RegistrationStep::default());
    
    // Step 1 signals (from Step 6)
    let referral_code = RwSignal::new(String::new());
    
    // Step 2 signals (NEW for Step 8)
    let email = RwSignal::new(String::new());
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let privacy_accepted = RwSignal::new(false);
    let eula_accepted = RwSignal::new(false);

    // URL parameter prefill (from Step 6)
    let query_params = use_query_map();
    Effect::new(move |_| {
        let params = query_params.get();
        if let Some(ref_code) = params.get("ref").or_else(|| params.get("referralUuid")) {
            if !ref_code.is_empty() {
                referral_code.set(ref_code.clone());
            }
        }
    });

    // Action for Step 6 → Step 7 (referral verification)
    let on_verify = Action::new(move |_code: &String| {
        let step = current_step;
        async move {
            step.set(RegistrationStep::Registration);
        }
    });

    // Action for Step 8 → Step 9 (registration submission)
    // In Step 8 this just advances to Success; Step 9 adds the real server call
    let on_register = Action::new(move |_: &()| {
        let step = current_step;
        async move {
            step.set(RegistrationStep::Success);
        }
    });

    view! {
        <div class="container large_font">
            // ... container_left (phone mockup) from Step 6 ...

            <div class="container_right">
                <div class="container_inner">
                    <div class="top_head_area">
                        // Back button
                        <a href="/login" class="btn btn-secondary back-btn" id="backBtn">
                            <span aria-hidden="true">
                                <i class="peer-icon medium_font peer-icon-arrow-left"></i>
                            </span>
                            "Back"
                        </a>
                    </div>

                    <div class="center_area">
                        // Step 1: Referral (from Step 6)
                        <div class=move || {
                            if current_step.get() == RegistrationStep::Referral {
                                "form-step active"
                            } else {
                                "form-step"
                            }
                        }>
                            <ReferralStep
                                referral_code=referral_code
                                on_verify=on_verify
                            />
                        </div>

                        // Step 2: Registration Form (NEW for Step 8)
                        <div class=move || {
                            if current_step.get() == RegistrationStep::Registration {
                                "form-step active"
                            } else {
                                "form-step"
                            }
                        }>
                            <RegistrationStep
                                email=email
                                username=username
                                password=password
                                confirm_password=confirm_password
                                privacy_accepted=privacy_accepted
                                eula_accepted=eula_accepted
                                on_submit=on_register
                            />
                        </div>

                        // Step 3: Success (placeholder — completed in Step 10)
                        <div class=move || {
                            if current_step.get() == RegistrationStep::Success {
                                "form-step active"
                            } else {
                                "form-step"
                            }
                        }>
                            <div class="step-header">
                                <h2 class="x_large_font">"Welcome to "<strong>"peer!"</strong></h2>
                                <p class="large_font">"Registration successful!"</p>
                            </div>
                        </div>
                    </div>

                    <div class="footer_area medium_font">
                        <p class="version version-number"></p>
                    </div>
                </div>
            </div>
        </div>
    }
}
```

---

## 8.8 — Component Module Registration

### Update `src/components/mod.rs`

```rust
pub mod referral;
pub mod registration_form;
pub mod password_strength;
pub mod validation;
```

---

## 8.9 — Testing Checklist

### 8.9.1 — Unit Tests

Run `cargo test` to verify all validation functions pass:

```bash
cargo test --lib
```

Expected output: All tests in `validation.rs` pass.

### 8.9.2 — Manual Testing Scenarios

| # | Test | Action | Expected |
|---|------|--------|----------|
| 1 | Email valid | Type `user@example.com` | Green check icon, no error message |
| 2 | Email invalid | Type `user@` | Red styling, "Please enter a valid email address" |
| 3 | Email empty | Clear field | Neutral styling, no error message |
| 4 | Username valid | Type `peer_user` | Green check icon |
| 5 | Username too short | Type `ab` | "Username must be 3-23 characters" |
| 6 | Username invalid chars | Type `user@name` | Error message shown |
| 7 | Password very weak | Type `abc` | Meter shows "Very weak", red colour |
| 8 | Password weak | Type `abcdefgh` | Meter shows "Weak" |
| 9 | Password good | Type `Abcd1234` | Meter shows "Good", green colour |
| 10 | Password excellent | Type `Abcd1234!` | Meter shows "Excellent" |
| 11 | Requirements hide | Type `Abcd1234` | All requirement items disappear |
| 12 | Password toggle | Click eye icon | Password becomes visible/hidden |
| 13 | Confirm matches | Type same password | Green check icon |
| 14 | Confirm mismatch | Type different password | "Passwords do not match" |
| 15 | Both checkboxes unchecked | Click submit | "Please accept both..." message |
| 16 | Only privacy checked | Click submit | "Please accept the EULA..." message |
| 17 | Both checked + valid | Click submit | Advances to step 3 (success placeholder) |
| 18 | Clear checkbox error | Check missing checkbox | Error message clears |

### 8.9.3 — Browser Testing

1. **Chrome DevTools Responsive Mode:** Test at 375px (mobile), 768px (tablet), 1440px (desktop)
2. **Form autofill:** Verify `autocomplete` attributes work (email, username, new-password)
3. **Tab navigation:** Ensure logical tab order through all fields
4. **Password visibility:** Both password fields toggle independently

---

## 8.10 — Common Pitfalls

| Pitfall | Solution |
|---------|----------|
| Password meter doesn't update | Ensure `password_validation` is a `Memo`, not a plain closure |
| Checkbox state not reactive | Use `prop:checked` with `event_target_checked()`, not `attr:checked` |
| Validation fires on empty fields | Check for empty string before showing error messages |
| Icon fonts not loading | Verify `peer-network.css` is imported and font files are served |
| Submit button doesn't work | Check that `on:submit` uses `ev.prevent_default()` |

---

## 8.11 — Definition of Done

Step 8 is complete when:

- [ ] `cargo check` and `cargo test --lib` pass
- [ ] Email field validates with correct regex matching JS
- [ ] Username field validates 3–23 chars, alphanumeric + `_-`
- [ ] Password strength meter shows all 5 levels correctly
- [ ] Password requirements list hides met requirements
- [ ] Password visibility toggle works for both fields
- [ ] Confirm password shows mismatch error when different
- [ ] Both checkboxes have error states and clear properly
- [ ] Form submission validates all fields before dispatching
- [ ] All manual test scenarios (8.9.2) pass
- [ ] DOM structure matches `register.php` for CSS parity

---

## 8.12 — Next Steps

**Step 9 — Registration Submission (Server Round-Trip)**

With the form UI complete, Step 9 will:
1. Replace the placeholder `on_register` action with the real `register_user` server function call
2. Handle loading state on the submit button
3. Map response codes (`10601`, `30601`, etc.) to user-friendly messages
4. Display backend validation errors on specific fields (e.g., duplicate email)

**Step 11 — Navigation & Back Button**

Step 11 will enhance the back button to:
1. Navigate from step 2 back to step 1 (preserving referral code)
2. Preserve form data when navigating back from step 3
3. Focus management on step transitions
