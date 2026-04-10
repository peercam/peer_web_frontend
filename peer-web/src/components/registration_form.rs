//! Registration form component (Step 2 of the registration flow).
//!
//! Renders email, username, password, confirm-password, and checkbox
//! fields with real-time client-side validation. The actual server
//! round-trip is wired in Step 9.

use leptos::prelude::*;
use leptos::web_sys;

use crate::components::password_strength::PasswordStrengthMeter;
use crate::components::validation::{
    is_valid_email, is_valid_username, passwords_match, validate_password,
};

/// Registration form with all input fields and client-side validation.
///
/// The parent (`RegisterPage`) owns the signals so that form data
/// persists when navigating back from step 3.
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
    on_submit: Action<(), ()>,
) -> impl IntoView {
    // --- Password visibility toggles ---
    let show_password = RwSignal::new(false);
    let show_confirm_password = RwSignal::new(false);

    // --- Derived validation states ---

    // Email validation
    let is_email_valid = Memo::new(move |_| is_valid_email(&email.get()));
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
    let is_username_valid = Memo::new(move |_| is_valid_username(&username.get()));
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
    let password_validation = Memo::new(move |_| validate_password(&password.get()));
    let is_password_valid = Memo::new(move |_| {
        password_validation.get().requirements.is_sufficient()
    });
    let password_visible = Memo::new(move |_| !password.get().is_empty());
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
    let checkbox_error_shown = Memo::new(move |_| !checkbox_message.get().is_empty());

    // Overall form validity (for submit button)
    let is_form_valid = Memo::new(move |_| {
        is_email_valid.get()
            && is_username_valid.get()
            && is_password_valid.get()
            && is_confirm_valid.get()
            && privacy_accepted.get()
            && eula_accepted.get()
    });

    // Handle form submission
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        // Validate checkboxes (set message if not checked)
        if !privacy_accepted.get() && !eula_accepted.get() {
            checkbox_message.set(
                "Please accept both the Privacy Policy and EULA".to_string(),
            );
            return;
        } else if !privacy_accepted.get() {
            checkbox_message.set(
                "Please accept the Privacy Policy to continue".to_string(),
            );
            return;
        } else if !eula_accepted.get() {
            checkbox_message.set(
                "Please accept the End User License Agreement (EULA) to continue".to_string(),
            );
            return;
        }

        // Clear checkbox message if both are checked
        checkbox_message.set(String::new());

        // Validate all fields
        if !is_form_valid.get() {
            return;
        }

        // Dispatch the submit action
        on_submit.dispatch(());
    };

    view! {
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
                                // Clear error when both checkboxes are checked
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
                                // Clear error when both checkboxes are checked
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
    }
}
