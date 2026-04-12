//! Login form component with email/password fields,
//! remember-me checkbox, and validation.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::components::validation::is_valid_email;
use crate::models::auth::LoginResponseCode;
use crate::state::auth::use_auth;
use crate::utils::cookies::set_remember_me;

/// Login form with email, password, remember-me, and submit button.
#[component]
pub fn LoginForm() -> impl IntoView {
    let auth = use_auth();
    let query = use_query_map();

    // Get redirect destination from query params (validated to start with /)
    let redirect_to = Memo::new(move |_| {
        query
            .get()
            .get("redirect")
            .filter(|p| p.starts_with('/'))
            .unwrap_or("/dashboard".to_string())
    });

    // Form signals
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let remember_me = RwSignal::new(true); // default checked
    let show_password = RwSignal::new(false);

    // Validation states (shown after blur)
    let email_touched = RwSignal::new(false);
    let password_touched = RwSignal::new(false);

    // Derived validation
    let is_email_valid = Memo::new(move |_| is_valid_email(&email.get()));
    let has_email_error = Memo::new(move |_| {
        email_touched.get() && !email.get().is_empty() && !is_email_valid.get()
    });
    let email_field_class = Memo::new(move |_| {
        if has_email_error.get() {
            "input-field invalid"
        } else if !email.get().is_empty() && is_email_valid.get() {
            "input-field valid"
        } else {
            "input-field"
        }
    });

    let has_password_error = Memo::new(move |_| {
        password_touched.get() && password.get().is_empty()
    });
    let password_field_class = Memo::new(move |_| {
        if has_password_error.get() {
            "input-field invalid"
        } else if !password.get().is_empty() {
            "input-field valid"
        } else {
            "input-field"
        }
    });

    // Form validity
    let is_form_valid = Memo::new(move |_| {
        is_email_valid.get() && !password.get().is_empty()
    });

    // Loading state
    let is_loading = Signal::derive(move || auth.login_action.pending().get());

    // Server error message (shown below form)
    let server_error = RwSignal::new(Option::<String>::None);

    // Handle login results
    Effect::new(move |_| {
        if let Some(result) = auth.login_action.value().get() {
            match result {
                Ok(payload) => {
                    let code = LoginResponseCode::from(
                        payload.code().unwrap_or(""),
                    );
                    match code {
                        LoginResponseCode::Success => {
                            // Persist remember-me preference
                            set_remember_me(remember_me.get_untracked());
                            // Redirect to original destination or dashboard
                            redirect_to_path(&redirect_to.get_untracked());
                        }
                        _ => {
                            server_error.set(Some(code.user_message().to_string()));
                        }
                    }
                }
                Err(e) => {
                    server_error.set(Some(
                        "Connection error. Please check your network and try again.".to_string()
                    ));
                    leptos::logging::error!("Login error: {:?}", e);
                }
            }
        }
    });

    // Submit handler
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        server_error.set(None);

        if !is_form_valid.get_untracked() {
            email_touched.set(true);
            password_touched.set(true);
            return;
        }

        auth.login_action.dispatch((
            email.get_untracked(),
            password.get_untracked(),
        ));
    };

    view! {
        <form
            class="form-container"
            on:submit=on_submit
            novalidate
        >
            // Server error message
            <Show when=move || server_error.get().is_some()>
                <div class="error-message form-error" role="alert">
                    {move || server_error.get().unwrap_or_default()}
                </div>
            </Show>

            // Email field
            <div class=move || email_field_class.get()>
                <label for="loginEmail" class="sr-only">"Email"</label>
                <div class="input-wrapper">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-email"></i>
                    </span>
                    <input
                        type="email"
                        id="loginEmail"
                        name="email"
                        placeholder="Enter your email"
                        autocomplete="email"
                        required
                        aria-required="true"
                        aria-invalid=move || has_email_error.get()
                        aria-describedby="loginEmailError"
                        prop:value=move || email.get()
                        on:input=move |ev| {
                            email.set(event_target_value(&ev));
                            server_error.set(None);
                        }
                        on:blur=move |_| email_touched.set(true)
                    />
                    <Show when=move || !email.get().is_empty() && is_email_valid.get()>
                        <span class="validation-icon valid" aria-hidden="true">
                            <i class="peer-icon peer-icon-check"></i>
                        </span>
                    </Show>
                </div>
                <Show when=move || has_email_error.get()>
                    <p class="error-text" id="loginEmailError" role="alert">
                        "Please enter a valid email address"
                    </p>
                </Show>
            </div>

            // Password field
            <div class=move || password_field_class.get()>
                <label for="loginPassword" class="sr-only">"Password"</label>
                <div class="input-wrapper">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-lock"></i>
                    </span>
                    <input
                        type=move || if show_password.get() { "text" } else { "password" }
                        id="loginPassword"
                        name="password"
                        placeholder="Enter your password"
                        autocomplete="current-password"
                        required
                        aria-required="true"
                        aria-invalid=move || has_password_error.get()
                        aria-describedby="loginPasswordError"
                        prop:value=move || password.get()
                        on:input=move |ev| {
                            password.set(event_target_value(&ev));
                            server_error.set(None);
                        }
                        on:blur=move |_| password_touched.set(true)
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
                                "peer-icon peer-icon-eye-off"
                            } else {
                                "peer-icon peer-icon-eye"
                            }
                        }></i>
                    </button>
                </div>
                <Show when=move || has_password_error.get()>
                    <p class="error-text" id="loginPasswordError" role="alert">
                        "Please enter your password"
                    </p>
                </Show>
            </div>

            // Remember me + Forgot password row
            <div class="form-options">
                <label class="checkbox-label">
                    <input
                        type="checkbox"
                        name="rememberMe"
                        prop:checked=move || remember_me.get()
                        on:change=move |ev| {
                            remember_me.set(event_target_checked(&ev));
                        }
                    />
                    " Remember me"
                </label>
                <a href="/forgotpassword" class="forgot-link">
                    "Forgot password?"
                </a>
            </div>

            // Submit button
            <button
                type="submit"
                class="btn btn-primary submit-btn"
                disabled=move || is_loading.get()
                aria-busy=move || is_loading.get()
            >
                <Show
                    when=move || is_loading.get()
                    fallback=|| view! { "Log In" }
                >
                    <span class="spinner" aria-hidden="true"></span>
                    " Logging in..."
                </Show>
            </button>

            // Link to register
            <p class="form-link">
                "Don't have an account? "
                <a href="/register">"Sign up"</a>
            </p>
        </form>
    }
}

/// Helper to get the checked state from a checkbox event.
fn event_target_checked(ev: &leptos::ev::Event) -> bool {
    use leptos::wasm_bindgen::JsCast;
    ev.target()
        .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
        .map(|el| el.checked())
        .unwrap_or(false)
}

/// Redirect to the specified path.
fn redirect_to_path(path: &str) {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href(path);
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = path;
    }
}
