//! Forgot password page — multi-step password reset flow.
//!
//! ## Step flow
//!
//! 1. Enter email → `request_password_reset`
//! 2. Verify code → `verify_reset_token`
//! 3. New password → `reset_password`
//! 4. Success → link to login
//!
//! ## Features
//!
//! - Rate-limited resend with escalating cooldowns (60s → 10min → locked)
//! - Email masking for privacy (`ca****@example.com`)
//! - Password strength meter
//! - Accessible: ARIA live regions, step announcements

use leptos::prelude::*;
use leptos::web_sys;
use leptos_meta::*;

use crate::api::forgot_password::{request_password_reset, reset_password, verify_reset_token};
use crate::components::left_panel::LeftPanel;
use crate::components::password_strength::PasswordStrengthMeter;
use crate::components::step_announcer::StepAnnouncer;
use crate::components::toast::{use_toast, ToastType};
use crate::components::validation::{is_valid_email, passwords_match, validate_password};
use crate::utils::response_codes::user_friendly_msg;

// ============================================================================
// Step Enum
// ============================================================================

/// Step identifier for the forgot password flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgotStep {
    /// Step 1: Enter email
    Email,
    /// Step 2: Verify code
    VerifyCode,
    /// Step 3: New password
    NewPassword,
    /// Step 4: Success
    Success,
}

impl ForgotStep {
    /// Returns the `data-step` attribute value.
    pub fn data_step(&self) -> &'static str {
        match self {
            Self::Email => "1",
            Self::VerifyCode => "2",
            Self::NewPassword => "3",
            Self::Success => "4",
        }
    }

    /// Screen reader announcement text for this step.
    pub fn announcement(&self) -> &'static str {
        match self {
            Self::Email => "Step 1: Enter your email address",
            Self::VerifyCode => "Step 2: Enter the verification code",
            Self::NewPassword => "Step 3: Create a new password",
            Self::Success => "Password updated successfully!",
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Mask an email address for display privacy.
///
/// `cameron@example.com` → `ca****@example.com`
fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((user, domain)) => {
            if user.len() <= 2 {
                format!("{}@{}", "*".repeat(user.len()), domain)
            } else {
                format!("{}{}@{}", &user[..2], "*".repeat(user.len() - 2), domain)
            }
        }
        None => email.to_string(),
    }
}

/// Navigate to a URL.
fn navigate_to(url: &str) {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(url);
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = url;
    }
}

// ============================================================================
// Main Page Component
// ============================================================================

/// The forgot password page component.
#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    // Step state
    let current_step = RwSignal::new(ForgotStep::Email);

    // Form data signals
    let email = RwSignal::new(String::new());
    let verify_code = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // Token saved from Step 2 for use in Step 3
    let reset_token = RwSignal::new(String::new());

    // Loading state
    let pending = RwSignal::new(false);

    // Announcement for screen readers
    let announcement = RwSignal::new(String::new());

    // Toast
    let toast = use_toast();

    // Resend cooldown state
    let countdown_seconds = RwSignal::new(0i32);
    let resend_count = RwSignal::new(0u32);

    // Back button handler
    let handle_back = move |_: web_sys::MouseEvent| {
        match current_step.get() {
            ForgotStep::Email => {
                navigate_to("/login");
            }
            ForgotStep::VerifyCode => {
                current_step.set(ForgotStep::Email);
                announcement.set(ForgotStep::Email.announcement().into());
            }
            ForgotStep::NewPassword => {
                current_step.set(ForgotStep::VerifyCode);
                announcement.set(ForgotStep::VerifyCode.announcement().into());
            }
            ForgotStep::Success => {}
        }
    };

    // Back button visibility
    let show_back = Memo::new(move |_| current_step.get() != ForgotStep::Success);

    // Back button href (Some for login, None for internal navigation)
    let back_href = Memo::new(move |_| {
        if current_step.get() == ForgotStep::Email {
            Some("/login".to_string())
        } else {
            None
        }
    });

    view! {
        <Title text="Peer Network - Forgot Password"/>
        <Meta name="description" content="Reset your Peer Network account password."/>

        // Skip navigation link
        <a href="#forgot-form" class="skip-link sr-only">
            "Skip to reset form"
        </a>

        <div class="container large_font">
            // Left panel: phone mockup
            <LeftPanel
                image_src="/img/register.webp"
                image_alt="Password reset"
            />

            // Right panel: multi-step form
            <div class="container_right">
                <div class="container_inner">
                    // Top area: back button
                    <div class="top_head_area">
                        <a
                            href=move || back_href.get().unwrap_or_default()
                            class="btn btn-secondary back-btn"
                            id="backBtn"
                            style:display=move || if show_back.get() { "flex" } else { "none" }
                            on:click=handle_back
                        >
                            <span aria-hidden="true">
                                <i class="peer-icon medium_font peer-icon-arrow-left"></i>
                            </span>
                            "Back"
                        </a>
                    </div>

                    // Center area: form steps
                    <div class="center_area" id="forgot-form" tabindex="-1">
                        <StepAnnouncer announcement=announcement.into()/>

                        // Step 1: Email
                        <EmailStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::Email)
                            email=email
                            pending=pending
                            toast=toast
                            on_success=move || {
                                current_step.set(ForgotStep::VerifyCode);
                                announcement.set(ForgotStep::VerifyCode.announcement().into());
                            }
                        />

                        // Step 2: Verify Code
                        <VerifyCodeStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::VerifyCode)
                            email=email.into()
                            verify_code=verify_code
                            pending=pending
                            toast=toast
                            countdown_seconds=countdown_seconds
                            resend_count=resend_count
                            on_success=move |token: String| {
                                reset_token.set(token);
                                current_step.set(ForgotStep::NewPassword);
                                announcement.set(ForgotStep::NewPassword.announcement().into());
                            }
                        />

                        // Step 3: New Password
                        <NewPasswordStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::NewPassword)
                            password=password
                            confirm_password=confirm_password
                            reset_token=reset_token.into()
                            pending=pending
                            toast=toast
                            on_success=move || {
                                current_step.set(ForgotStep::Success);
                                announcement.set(ForgotStep::Success.announcement().into());
                            }
                        />

                        // Step 4: Success
                        <SuccessStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::Success)
                        />
                    </div>

                    // Footer area
                    <div class="footer_area medium_font">
                        <p class="version version-number"></p>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// Step 1: Email Input
// ============================================================================

#[component]
fn EmailStep(
    /// Whether this step is currently active.
    active: Signal<bool>,
    /// Email input signal.
    email: RwSignal<String>,
    /// Loading state.
    pending: RwSignal<bool>,
    /// Toast context for notifications.
    toast: crate::components::toast::ToastContext,
    /// Callback when email is submitted successfully.
    on_success: impl Fn() + Clone + 'static,
) -> impl IntoView {
    // Email validation
    let is_email_valid = Memo::new(move |_| is_valid_email(&email.get()));
    let email_touched = RwSignal::new(false);

    // Backend error display
    let backend_error = RwSignal::new(Option::<String>::None);

    // Field class based on validation state
    let email_field_class = Memo::new(move |_| {
        let e = email.get();
        if e.is_empty() {
            "input-field"
        } else if is_email_valid.get() && backend_error.get().is_none() {
            "input-field valid"
        } else {
            "input-field invalid"
        }
    });

    // Error message
    let error_message = Memo::new(move |_| {
        if let Some(err) = backend_error.get() {
            return err;
        }
        let e = email.get();
        if !email_touched.get() || e.is_empty() {
            String::new()
        } else if !is_email_valid.get() {
            "Please enter a valid email address".to_string()
        } else {
            String::new()
        }
    });

    // Submit handler
    let on_success = on_success.clone();
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        email_touched.set(true);
        backend_error.set(None);

        if !is_email_valid.get() {
            return;
        }

        let email_value = email.get();
        let on_success = on_success.clone();

        pending.set(true);
        leptos::task::spawn_local(async move {
            match request_password_reset(email_value).await {
                Ok(payload) if payload.is_success() => {
                    toast.show(
                        "If an account exists, you'll receive an email with instructions.",
                        ToastType::Success,
                    );
                    on_success();
                }
                Ok(payload) => {
                    let code = payload.response_code.unwrap_or_default();
                    match code.as_str() {
                        "31901" | "31903" => {
                            backend_error.set(Some(user_friendly_msg(&code).to_string()));
                        }
                        _ => {
                            toast.show(user_friendly_msg(&code), ToastType::Error);
                        }
                    }
                }
                Err(e) => {
                    toast.show(&e.to_string(), ToastType::Error);
                }
            }
            pending.set(false);
        });
    };

    view! {
        <div
            class=move || if active.get() { "form-step active" } else { "form-step" }
            data-step="1"
            id="emailStep"
        >
            <div class="step-header">
                <h2 class="x_large_font">"Forgot password"</h2>
                <p class="large_font">
                    "Please enter your email and we'll send you a reset link."
                </p>
            </div>

            <form on:submit=handle_submit novalidate=true>
                // Email input
                <div class="input-group">
                    <div class=move || email_field_class.get() id="emailField">
                        <span class="input-icon" aria-hidden="true">
                            <i class="peer-icon peer-icon-email"></i>
                        </span>
                        <input
                            type="email"
                            id="email"
                            name="email"
                            placeholder="Email"
                            autocomplete="email"
                            required=true
                            aria-required="true"
                            aria-invalid=move || (!email.get().is_empty() && !is_email_valid.get()).to_string()
                            aria-describedby="emailError"
                            prop:value=move || email.get()
                            on:input=move |ev| {
                                email.set(event_target_value(&ev));
                                backend_error.set(None);
                            }
                            on:blur=move |_| email_touched.set(true)
                        />
                        <span class="valid-icon" aria-hidden="true">
                            <Show when=move || is_email_valid.get() && !email.get().is_empty() && backend_error.get().is_none()>
                                <i class="peer-icon peer-icon-good-tick-circle"></i>
                            </Show>
                        </span>
                    </div>
                    <p
                        class="error-message medium_font"
                        id="emailError"
                        role="alert"
                        aria-live="polite"
                    >
                        {move || error_message.get()}
                    </p>
                </div>

                // Submit button
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || pending.get() || !is_email_valid.get()
                    aria-busy=move || pending.get().to_string()
                >
                    <Show when=move || pending.get() fallback=|| "Reset password">
                        <span class="spinner"></span>
                        " Sending..."
                    </Show>
                </button>
            </form>
        </div>
    }
}

// ============================================================================
// Step 2: Verify Code
// ============================================================================

#[component]
fn VerifyCodeStep(
    /// Whether this step is currently active.
    active: Signal<bool>,
    /// Email for masked display.
    email: Signal<String>,
    /// Verification code input signal.
    verify_code: RwSignal<String>,
    /// Loading state.
    pending: RwSignal<bool>,
    /// Toast context.
    toast: crate::components::toast::ToastContext,
    /// Countdown timer seconds.
    countdown_seconds: RwSignal<i32>,
    /// Number of resend attempts.
    resend_count: RwSignal<u32>,
    /// Callback when code is verified; receives the token.
    on_success: impl Fn(String) + Clone + 'static,
) -> impl IntoView {
    // Masked email
    let masked_email = Memo::new(move |_| mask_email(&email.get()));

    // Code validation (non-empty)
    let is_code_valid = Memo::new(move |_| !verify_code.get().trim().is_empty());

    // Backend error
    let backend_error = RwSignal::new(Option::<String>::None);

    // Field class
    let code_field_class = Memo::new(move |_| {
        let c = verify_code.get();
        if c.is_empty() {
            "input-field"
        } else if backend_error.get().is_some() {
            "input-field invalid"
        } else {
            "input-field valid"
        }
    });

    // Countdown display (MM:SS)
    let countdown_display = Memo::new(move |_| {
        let secs = countdown_seconds.get();
        if secs <= 0 {
            String::new()
        } else {
            format!("{:02}:{:02}", secs / 60, secs % 60)
        }
    });

    // Can resend: countdown finished AND not locked (3+ attempts)
    let is_locked = Memo::new(move |_| resend_count.get() >= 3);
    let has_countdown = Memo::new(move |_| countdown_seconds.get() > 0);
    let can_resend = Memo::new(move |_| !has_countdown.get() && !is_locked.get());

    // Start countdown timer
    let start_countdown = move |duration: i32| {
        countdown_seconds.set(duration);

        #[cfg(feature = "hydrate")]
        {
            use gloo_timers::callback::Interval;

            let interval = Interval::new(1_000, move || {
                countdown_seconds.update(|s| {
                    if *s > 0 {
                        *s -= 1;
                    }
                });
            });

            // Store interval handle to prevent early drop
            leptos::on_cleanup(move || drop(interval));
        }
    };

    // Submit handler
    let on_success = on_success.clone();
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        backend_error.set(None);

        let code_value = verify_code.get().trim().to_string();
        if code_value.is_empty() {
            backend_error.set(Some("Please enter the verification code".to_string()));
            return;
        }

        let on_success = on_success.clone();

        pending.set(true);
        leptos::task::spawn_local(async move {
            match verify_reset_token(code_value.clone()).await {
                Ok(payload) if payload.is_success() => {
                    toast.show("Code verified.", ToastType::Success);
                    on_success(code_value);
                }
                Ok(payload) => {
                    let code = payload.response_code.unwrap_or_default();
                    backend_error.set(Some(user_friendly_msg(&code).to_string()));
                }
                Err(e) => {
                    backend_error.set(Some(e.to_string()));
                }
            }
            pending.set(false);
        });
    };

    // Resend handler
    let handle_resend = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();

        if !can_resend.get() {
            return;
        }

        let email_value = email.get();
        let count = resend_count.get();

        pending.set(true);
        leptos::task::spawn_local(async move {
            match request_password_reset(email_value).await {
                Ok(payload) if payload.is_success() => {
                    toast.show("Code resent. Check your email.", ToastType::Success);
                    resend_count.set(count + 1);

                    // Escalating cooldowns
                    let cooldown = match count {
                        0 => 60,    // First resend: 60 seconds
                        1 => 600,   // Second resend: 10 minutes
                        _ => 0,     // 3+ should be locked
                    };
                    if cooldown > 0 {
                        start_countdown(cooldown);
                    }
                }
                Ok(payload) => {
                    let code = payload.response_code.unwrap_or_default();
                    toast.show(user_friendly_msg(&code), ToastType::Error);
                }
                Err(e) => {
                    toast.show(&e.to_string(), ToastType::Error);
                }
            }
            pending.set(false);
        });
    };

    view! {
        <div
            class=move || if active.get() { "form-step active" } else { "form-step" }
            data-step="2"
            id="verifyCodeStep"
        >
            <div class="step-header">
                <h2 class="x_large_font">"Verify code"</h2>
                <p class="large_font">
                    "We sent a code to "
                    <strong>{move || masked_email.get()}</strong>
                    ". Enter it below."
                </p>
            </div>

            <form on:submit=handle_submit novalidate=true>
                // Code input
                <div class="input-group">
                    <div class=move || code_field_class.get() id="codeField">
                        <span class="input-icon" aria-hidden="true">
                            <i class="peer-icon peer-icon-key"></i>
                        </span>
                        <input
                            type="text"
                            id="verifyCode"
                            name="verifyCode"
                            placeholder="Verification code"
                            autocomplete="one-time-code"
                            inputmode="text"
                            required=true
                            aria-required="true"
                            aria-invalid=move || backend_error.get().is_some().to_string()
                            aria-describedby="codeError"
                            prop:value=move || verify_code.get()
                            on:input=move |ev| {
                                verify_code.set(event_target_value(&ev));
                                backend_error.set(None);
                            }
                        />
                        <span class="valid-icon" aria-hidden="true">
                            <Show when=move || is_code_valid.get() && backend_error.get().is_none()>
                                <i class="peer-icon peer-icon-good-tick-circle"></i>
                            </Show>
                        </span>
                    </div>
                    <p
                        class="error-message medium_font"
                        id="codeError"
                        role="alert"
                        aria-live="polite"
                    >
                        {move || backend_error.get().unwrap_or_default()}
                    </p>
                </div>

                // Submit button
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || pending.get() || !is_code_valid.get()
                    aria-busy=move || pending.get().to_string()
                >
                    <Show when=move || pending.get() fallback=|| "Verify code">
                        <span class="spinner"></span>
                        " Verifying..."
                    </Show>
                </button>
            </form>

            // Resend section
            <div class="dont_get_code medium_font">
                <p>
                    "Didn't get the code? "
                    <Show when=move || has_countdown.get()>
                        <span class="resend-txt">
                            "You can resend in "
                            <span class="countdown">{move || countdown_display.get()}</span>
                        </span>
                    </Show>
                    <Show when=move || can_resend.get()>
                        <a
                            href="#"
                            class="resend-link"
                            on:click=handle_resend
                        >
                            "Resend Code"
                        </a>
                    </Show>
                    <Show when=move || is_locked.get()>
                        <span class="error-text">
                            "Email delivery failed. Please contact support at "
                            <a href="mailto:peernetworkpse@gmail.com">"peernetworkpse@gmail.com"</a>
                        </span>
                    </Show>
                </p>
            </div>
        </div>
    }
}

// ============================================================================
// Step 3: New Password
// ============================================================================

#[component]
fn NewPasswordStep(
    /// Whether this step is currently active.
    active: Signal<bool>,
    /// Password input signal.
    password: RwSignal<String>,
    /// Confirm password input signal.
    confirm_password: RwSignal<String>,
    /// Reset token from Step 2.
    reset_token: Signal<String>,
    /// Loading state.
    pending: RwSignal<bool>,
    /// Toast context.
    toast: crate::components::toast::ToastContext,
    /// Callback when password is updated.
    on_success: impl Fn() + Clone + 'static,
) -> impl IntoView {
    // Password visibility toggles
    let show_password = RwSignal::new(false);
    let show_confirm = RwSignal::new(false);

    // Password validation
    let password_validation = Memo::new(move |_| validate_password(&password.get()));
    let is_password_valid = Memo::new(move |_| {
        password_validation.get().requirements.is_sufficient()
    });
    let password_visible = Memo::new(move |_| !password.get().is_empty());

    // Confirm password validation
    let is_confirm_valid = Memo::new(move |_| {
        passwords_match(&password.get(), &confirm_password.get())
    });

    // Backend error
    let backend_error = RwSignal::new(Option::<String>::None);

    // Field classes
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

    // Confirm error message
    let confirm_message = Memo::new(move |_| {
        let c = confirm_password.get();
        if c.is_empty() || is_confirm_valid.get() {
            String::new()
        } else {
            "Passwords do not match".to_string()
        }
    });

    // Form validity
    let can_submit = Memo::new(move |_| {
        is_password_valid.get() && is_confirm_valid.get() && !pending.get()
    });

    // Submit handler
    let on_success = on_success.clone();
    let handle_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        backend_error.set(None);

        if !can_submit.get() {
            return;
        }

        let token = reset_token.get();
        let password_value = password.get();
        let on_success = on_success.clone();

        pending.set(true);
        leptos::task::spawn_local(async move {
            match reset_password(token, password_value).await {
                Ok(payload) if payload.is_success() => {
                    toast.show("Password updated successfully!", ToastType::Success);
                    on_success();
                }
                Ok(payload) => {
                    let code = payload.response_code.unwrap_or_default();
                    backend_error.set(Some(user_friendly_msg(&code).to_string()));
                }
                Err(e) => {
                    backend_error.set(Some(e.to_string()));
                }
            }
            pending.set(false);
        });
    };

    view! {
        <div
            class=move || if active.get() { "form-step active" } else { "form-step" }
            data-step="3"
            id="newPasswordStep"
        >
            <div class="step-header">
                <h2 class="x_large_font">"Enter new password"</h2>
            </div>

            <form on:submit=handle_submit novalidate=true>
                // Password input
                <div class="input-group">
                    <div class=move || password_field_class.get() id="passwordField">
                        <span class="input-icon" aria-hidden="true">
                            <i class="peer-icon peer-icon-lock"></i>
                        </span>
                        <input
                            type=move || if show_password.get() { "text" } else { "password" }
                            id="password"
                            name="password"
                            placeholder="New password"
                            autocomplete="new-password"
                            required=true
                            aria-required="true"
                            aria-invalid=move || (!password.get().is_empty() && !is_password_valid.get()).to_string()
                            aria-describedby="passwordError passwordStrength"
                            prop:value=move || password.get()
                            on:input=move |ev| {
                                password.set(event_target_value(&ev));
                                backend_error.set(None);
                            }
                        />
                        <button
                            type="button"
                            class="toggle-password"
                            aria-label=move || {
                                if show_password.get() {
                                    "Hide password"
                                } else {
                                    "Show password"
                                }
                            }
                            on:click=move |_| show_password.update(|v| *v = !*v)
                        >
                            <i class=move || {
                                if show_password.get() {
                                    "peer-icon peer-icon-eye-hidden"
                                } else {
                                    "peer-icon peer-icon-eye-visible"
                                }
                            }></i>
                        </button>
                    </div>
                </div>

                // Password strength meter
                <PasswordStrengthMeter
                    validation=password_validation
                    visible=password_visible
                />

                // Confirm password input
                <div class="input-group">
                    <div class=move || confirm_field_class.get() id="confirmPasswordField">
                        <span class="input-icon" aria-hidden="true">
                            <i class="peer-icon peer-icon-lock"></i>
                        </span>
                        <input
                            type=move || if show_confirm.get() { "text" } else { "password" }
                            id="confirmPassword"
                            name="confirmPassword"
                            placeholder="Confirm password"
                            autocomplete="new-password"
                            required=true
                            aria-required="true"
                            aria-invalid=move || (!confirm_password.get().is_empty() && !is_confirm_valid.get()).to_string()
                            aria-describedby="confirmError"
                            prop:value=move || confirm_password.get()
                            on:input=move |ev| confirm_password.set(event_target_value(&ev))
                        />
                        <button
                            type="button"
                            class="toggle-password"
                            aria-label=move || {
                                if show_confirm.get() {
                                    "Hide password"
                                } else {
                                    "Show password"
                                }
                            }
                            on:click=move |_| show_confirm.update(|v| *v = !*v)
                        >
                            <i class=move || {
                                if show_confirm.get() {
                                    "peer-icon peer-icon-eye-hidden"
                                } else {
                                    "peer-icon peer-icon-eye-visible"
                                }
                            }></i>
                        </button>
                    </div>
                    <p
                        class="error-message medium_font"
                        id="confirmError"
                        role="alert"
                        aria-live="polite"
                    >
                        {move || confirm_message.get()}
                    </p>
                </div>

                // Backend error display
                <Show when=move || backend_error.get().is_some()>
                    <p class="error-message medium_font" role="alert">
                        {move || backend_error.get().unwrap_or_default()}
                    </p>
                </Show>

                // Submit button
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || !can_submit.get()
                    aria-busy=move || pending.get().to_string()
                >
                    <Show when=move || pending.get() fallback=|| "Update password">
                        <span class="spinner"></span>
                        " Updating..."
                    </Show>
                </button>
            </form>
        </div>
    }
}

// ============================================================================
// Step 4: Success
// ============================================================================

#[component]
fn SuccessStep(
    /// Whether this step is currently active.
    active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div
            class=move || if active.get() { "form-step active" } else { "form-step" }
            data-step="4"
            id="successStep"
        >
            <div class="success-message">
                <div class="step-header">
                    <span class="success-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-good-tick-circle"></i>
                    </span>
                    <h2 class="x_large_font">"Password updated!"</h2>
                    <p class="large_font">
                        "Your password has been updated. Please use your new password to log in."
                    </p>
                </div>
                <a class="btn btn-primary" href="/login">
                    "Continue to Login"
                </a>
            </div>
        </div>
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_email_standard() {
        assert_eq!(mask_email("cameron@example.com"), "ca*****@example.com");
    }

    #[test]
    fn mask_email_short_user() {
        assert_eq!(mask_email("ab@example.com"), "**@example.com");
    }

    #[test]
    fn mask_email_single_char() {
        assert_eq!(mask_email("a@example.com"), "*@example.com");
    }

    #[test]
    fn mask_email_no_at_sign() {
        assert_eq!(mask_email("notanemail"), "notanemail");
    }

    #[test]
    fn mask_email_empty() {
        assert_eq!(mask_email(""), "");
    }
}
