//! Registration page — multi-step shell component.
//!
//! This component renders the full page layout for `/register`:
//! - Left panel: phone mockup with image and animated logo
//! - Right panel: multi-step form container
//!
//! The actual form content for each step is a placeholder here.
//! Steps 6–10 will replace each placeholder with real components.
//!
//! ## Step flow
//!
//! 1. Referral code entry (step 1) / default referral (step 1b)
//! 2. Registration form (step 2)
//! 3. Success confirmation (step 3)

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_query_map;

use crate::api::registration::{register_user, verify_account, verify_referral};
use crate::components::back_button::BackButton;
use crate::components::referral::{DefaultReferralView, ReferralStep};
use crate::components::registration_form::{focus_field, RegistrationStep};
use crate::components::success_step::SuccessStep;
use crate::components::toast::{use_toast, ToastType};
use crate::models::user::ReferralUser;
use crate::utils::response_codes::user_friendly_msg;

/// Registration step identifier.
///
/// Tracks which step of the multi-step form is currently visible.
/// The CSS class `.active` is applied to the matching `.form-step` div.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegStep {
    /// Step 1: Enter referral code
    Referral,
    /// Step 1b: Show default referral code
    DefaultReferral,
    /// Step 2: Registration form (email, username, password)
    Register,
    /// Step 3: Success confirmation
    Success,
}

impl RegStep {
    /// Returns the `data-step` attribute value matching register.php.
    pub fn data_step(&self) -> &'static str {
        match self {
            Self::Referral => "1",
            Self::DefaultReferral => "1b",
            Self::Register => "2",
            Self::Success => "3",
        }
    }

    /// Internal step number for history hash fragments and ordering.
    pub fn number(&self) -> u8 {
        match self {
            Self::Referral | Self::DefaultReferral => 1,
            Self::Register => 2,
            Self::Success => 3,
        }
    }

    /// Screen reader announcement text for this step.
    pub fn announcement(&self) -> &'static str {
        match self {
            Self::Referral => "Step 1: Referral Code Entry",
            Self::DefaultReferral => "Step 1: Claim Your Invitation",
            Self::Register => "Step 2: Registration Form",
            Self::Success => "Registration successful! Welcome to peer!",
        }
    }

    /// Returns true if the back button should be visible for this step.
    pub fn show_back_button(&self) -> bool {
        match self {
            Self::Referral | Self::DefaultReferral | Self::Register => true,
            Self::Success => false,
        }
    }

    /// Returns the previous step for back-button navigation.
    /// `None` means navigate to `/login` (external).
    pub fn previous(&self) -> Option<RegStep> {
        match self {
            Self::Referral => None,
            Self::DefaultReferral => Some(Self::Referral),
            Self::Register => Some(Self::Referral),
            Self::Success => None,
        }
    }

    /// The DOM element ID for the step container.
    fn element_id(&self) -> &'static str {
        match self {
            Self::Referral => "referralStep",
            Self::DefaultReferral => "defaultReferralStep",
            Self::Register => "registrationStep",
            Self::Success => "successStep",
        }
    }
}

/// Focus the first interactive element within the active step container.
fn focus_first_interactive_in_step(step: RegStep) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;

        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(container) = document.get_element_by_id(step.element_id()) {
                let selector = "input, button, select, textarea, a[href]";
                if let Ok(Some(el)) = container.query_selector(selector) {
                    if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = html_el.focus();
                    }
                }
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = step;
    }
}

/// Wrapper for requestAnimationFrame to delay focus until after DOM update.
fn request_animation_frame(f: impl FnOnce() + 'static) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        let closure = Closure::once_into_js(f);
        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = f;
    }
}

/// Push the current step into the browser history via hash fragment.
fn push_step_to_history(step: RegStep) {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(history) = window.history() {
                let hash = format!("#step-{}", step.number());
                let _ = history.push_state_with_url(
                    &wasm_bindgen::JsValue::NULL,
                    "",
                    Some(&hash),
                );
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = step;
    }
}

/// Listen for browser popstate events (back/forward buttons)
/// and update `current_step` accordingly.
fn listen_for_popstate(current_step: RwSignal<RegStep>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        if let Some(window) = web_sys::window() {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Some(w) = web_sys::window() {
                    if let Ok(hash) = w.location().hash() {
                        let step = match hash.as_str() {
                            "#step-2" => RegStep::Register,
                            "#step-3" => RegStep::Success,
                            _ => RegStep::Referral,
                        };
                        current_step.set(step);
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback(
                "popstate",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget(); // leak intentionally — lives for page lifetime
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = current_step;
    }
}

/// The main registration page component.
///
/// Renders the full registration page layout with:
/// - Phone mockup (left panel)
/// - Multi-step form container (right panel)
/// - Reactive step transitions controlled by `current_step`
///
/// ## Query parameters
///
/// - `?ref=<UUID>` — Pre-fills the referral code input (read on mount)
#[component]
pub fn RegisterPage() -> impl IntoView {
    // ── Reactive state ──────────────────────────────────────────────────
    let current_step = RwSignal::new(RegStep::Referral);
    let referral_code = RwSignal::new(String::new());
    let verified_referrer = RwSignal::new(None::<ReferralUser>);

    // Step 2 signals
    let email = RwSignal::new(String::new());
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let privacy_accepted = RwSignal::new(false);
    let eula_accepted = RwSignal::new(false);

    // ── Toast context ───────────────────────────────────────────────────
    let toast = use_toast();

    // ── Backend error signals (distinct from client-side validation) ─────
    let email_backend_error = RwSignal::new(None::<String>);
    let username_backend_error = RwSignal::new(None::<String>);

    // ── Read ?ref= or ?referralUuid= query parameter on mount ──────────
    let query = use_query_map();
    Effect::new(move |_| {
        let params = query.get();
        if let Some(ref_code) = params.get("ref").or_else(|| params.get("referralUuid"))
            && !ref_code.is_empty()
        {
            referral_code.set(ref_code);
        }
    });

    // ── Helper: CSS class for a form step ───────────────────────────────
    let step_class = move |step: RegStep| {
        move || {
            if current_step.get() == step {
                "form-step active"
            } else {
                "form-step"
            }
        }
    };

    // ── Verify action: calls verify_referral server function ──────────
    let verify_action = Action::new(move |code: &String| {
        let code = code.clone();
        async move { verify_referral(code).await }
    });

    // Wrap the server action in a simple Action<String, ()> for the child component
    let on_verify = Action::new(move |code: &String| {
        let code = code.clone();
        verify_action.dispatch(code);
        async move {}
    });

    // Pending signal for loading state
    let verify_pending = Signal::derive(move || verify_action.pending().get());

    // ── Registration action: calls register_user server function ────────
    let register_action = Action::new({
        let email = email.clone();
        let password = password.clone();
        let username = username.clone();
        let referral_code = referral_code.clone();
        move |_: &()| {
            let email_val = email.get();
            let password_val = password.get();
            let username_val = username.get();
            let referral_val = referral_code.get();
            async move {
                register_user(email_val, password_val, username_val, referral_val).await
            }
        }
    });

    // Pending signal for registration loading state
    let register_pending = Signal::derive(move || register_action.pending().get());

    // The on_register action dispatched by the form clears backend errors, then calls register_action
    let on_register = Action::new(move |_: &()| {
        email_backend_error.set(None);
        username_backend_error.set(None);
        register_action.dispatch(());
        async move {}
    });

    // ── Verify account action (called after successful registration) ────
    let verify_acct_action = Action::new(move |userid: &String| {
        let userid = userid.clone();
        async move { verify_account(userid).await }
    });

    // ── Handle register_action results ──────────────────────────────────
    Effect::new(move |_| {
        if let Some(result) = register_action.value().get() {
            match result {
                Ok(response) => {
                    let code = response.code().unwrap_or("");
                    match code {
                        "10601" => {
                            // Success — call verify_account and advance
                            if let Some(userid) = &response.user_id {
                                verify_acct_action.dispatch(userid.clone());
                            }

                            toast.show(user_friendly_msg("10601"), ToastType::Success);
                            // SuccessStep component handles sessionStorage write on mount
                            current_step.set(RegStep::Success);
                        }
                        "30601" => {
                            // Duplicate email
                            email_backend_error
                                .set(Some(user_friendly_msg("30601").to_string()));
                            focus_field("email");
                        }
                        "30202" => {
                            // Invalid username
                            username_backend_error
                                .set(Some(user_friendly_msg("30202").to_string()));
                            focus_field("username");
                        }
                        other => {
                            toast.show(user_friendly_msg(other), ToastType::Error);
                        }
                    }
                }
                Err(e) => {
                    leptos::logging::error!("Registration error: {:?}", e);
                    toast.show(
                        "Connection error. Please check your network and try again.",
                        ToastType::Error,
                    );
                }
            }
        }
    });

    // ── Handle verify_acct_action results (logging only) ────────────────
    Effect::new(move |_| {
        if let Some(result) = verify_acct_action.value().get() {
            match result {
                Ok(response) => {
                    leptos::logging::log!("Account verification: {:?}", response.response_code);
                }
                Err(e) => {
                    leptos::logging::warn!("Account verification failed: {:?}", e);
                }
            }
        }
    });

    // ── Handle verify_action results ────────────────────────────────────
    Effect::new(move |_| {
        if let Some(result) = verify_action.value().get() {
            match result {
                Ok(response) => {
                    if response.is_success() {
                        // Store referrer info
                        if let Some(referrer) = response.referrer() {
                            verified_referrer.set(Some(referrer.clone()));
                        }

                        toast.show(
                            user_friendly_msg(&response.response_code),
                            ToastType::Success,
                        );

                        current_step.set(RegStep::Register);
                    } else {
                        toast.show(
                            user_friendly_msg(&response.response_code),
                            ToastType::Error,
                        );
                    }
                }
                Err(e) => {
                    toast.show(
                        format!("Error verifying referral code: {e}"),
                        ToastType::Error,
                    );
                }
            }
        }
    });

    // ── Back-button derived signals ────────────────────────────────────
    let show_back = Memo::new(move |_| current_step.get().show_back_button());

    let back_href = Memo::new(move |_| -> Option<String> {
        match current_step.get() {
            RegStep::Referral => Some("/login".to_string()),
            _ => None,
        }
    });

    let go_back = Callback::new(move |_: ()| {
        if let Some(prev) = current_step.get().previous() {
            current_step.set(prev);
        }
    });

    // ── Screen reader step announcement ──────────────────────────────
    let step_announcement = Memo::new(move |_| {
        current_step.get().announcement().to_string()
    });

    // ── Focus management: focus first interactive element on step change ──
    Effect::new(move |_| {
        let step = current_step.get();
        request_animation_frame(move || {
            focus_first_interactive_in_step(step);
        });
    });

    // ── Browser history integration ─────────────────────────────────────
    // On mount: read initial hash and set step accordingly
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(hash) = window.location().hash() {
                    let step = match hash.as_str() {
                        "#step-2" => RegStep::Register,
                        "#step-3" => RegStep::Success,
                        _ => RegStep::Referral,
                    };
                    if step != RegStep::Referral {
                        current_step.set(step);
                    }
                }
            }
        }
    });

    // Listen for browser back/forward
    listen_for_popstate(current_step);

    // Push step changes to history
    Effect::new(move |_| {
        let step = current_step.get();
        push_step_to_history(step);
    });

    // ── View ────────────────────────────────────────────────────────────
    view! {
        <Title text="Peer Network - Register"/>
        <Meta name="description" content="Create your Peer Network account. Join the blockchain-based social network."/>

        <div class="container large_font">
            // ── Left panel: phone mockup ────────────────────────────────
            <div class="container_left">
                <div class="phone">
                    <div class="screen">
                        <img
                            src="/img/register.webp"
                            alt="Register preview"
                            width="612"
                            height="612"
                        />
                    </div>
                    <div class="home-button">
                        <img
                            src="/svg/logo_sw.svg"
                            alt="Peer Logo"
                            width="96"
                            height="96"
                        />
                    </div>
                </div>
                <img
                    class="logo"
                    src="/svg/logo_farbe.svg"
                    alt="Peer logo"
                    width="96"
                    height="96"
                />
            </div>

            // ── Right panel: form container ─────────────────────────────
            <div class="container_right">
                <div class="container_inner">
                    // ── Top area: back button ───────────────────────────
                    <div class="top_head_area">
                        <BackButton
                            visible=show_back.into()
                            href=back_href.into()
                            on_back=go_back
                        />
                    </div>

                    // ── Center area: form steps ─────────────────────────
                    <div class="center_area">

                        // Step 1: Referral Code
                        <div
                            class=step_class(RegStep::Referral)
                            data-step="1"
                            id="referralStep"
                        >
                            <div class="step-header">
                                <h2 class="x_large_font">
                                    "Welcome to " <strong>"peer!"</strong>
                                </h2>
                                <p class="large_font">
                                    "One quick step left! Enter your referral code to complete registration."
                                </p>
                            </div>
                            <ReferralStep
                                referral_code=referral_code
                                on_verify=on_verify
                                pending=verify_pending
                                on_show_default=Callback::new(move |()| {
                                    current_step.set(RegStep::DefaultReferral);
                                })
                            />
                        </div>

                        // Step 1b: Default Referral Code
                        <div
                            class=step_class(RegStep::DefaultReferral)
                            data-step="1b"
                            id="defaultReferralStep"
                        >
                            <div class="step-header">
                                <h2 class="x_large_font">"Claim Your Invitation"</h2>
                                <p class="large_font">
                                    "Earning starts the moment you enter this magic code"
                                </p>
                            </div>
                            <DefaultReferralView
                                on_use_code=Callback::new(move |code: String| {
                                    referral_code.set(code);
                                    current_step.set(RegStep::Referral);
                                })
                            />
                        </div>

                        // Step 2: Registration Form
                        <div
                            class=step_class(RegStep::Register)
                            data-step="2"
                            id="registrationStep"
                        >
                            <div class="step-header">
                                <h2 class="x_large_font">"Register"</h2>
                                <p class="large_font">
                                    "Create your account in few seconds and start earning on your favorite content."
                                </p>
                            </div>
                            <RegistrationStep
                                email=email
                                username=username
                                password=password
                                confirm_password=confirm_password
                                privacy_accepted=privacy_accepted
                                eula_accepted=eula_accepted
                                email_backend_error=email_backend_error
                                username_backend_error=username_backend_error
                                pending=register_pending
                                on_submit=on_register
                            />
                        </div>

                        // Step 3: Success
                        <div
                            class=step_class(RegStep::Success)
                            data-step="3"
                            id="successStep"
                        >
                            <SuccessStep email=email.into() />
                        </div>
                    </div>

                    // ── Footer area ─────────────────────────────────────
                    <div class="footer_area medium_font">
                        <p class="version version-number"></p>
                    </div>
                </div>

                // ── Screen reader step announcer ────────────────────────
                <div
                    id="step-announcer"
                    class="sr-only"
                    aria-live="polite"
                    aria-atomic="true"
                >
                    {step_announcement}
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_step_values() {
        assert_eq!(RegStep::Referral.data_step(), "1");
        assert_eq!(RegStep::DefaultReferral.data_step(), "1b");
        assert_eq!(RegStep::Register.data_step(), "2");
        assert_eq!(RegStep::Success.data_step(), "3");
    }

    #[test]
    fn test_back_button_visible_on_step_1() {
        assert!(RegStep::Referral.show_back_button());
    }

    #[test]
    fn test_back_button_visible_on_step_1b() {
        assert!(RegStep::DefaultReferral.show_back_button());
    }

    #[test]
    fn test_back_button_visible_on_step_2() {
        assert!(RegStep::Register.show_back_button());
    }

    #[test]
    fn test_back_button_hidden_on_step_3() {
        assert!(!RegStep::Success.show_back_button());
    }

    #[test]
    fn test_previous_from_referral_is_none() {
        assert_eq!(RegStep::Referral.previous(), None);
    }

    #[test]
    fn test_previous_from_default_referral() {
        assert_eq!(RegStep::DefaultReferral.previous(), Some(RegStep::Referral));
    }

    #[test]
    fn test_previous_from_register() {
        assert_eq!(RegStep::Register.previous(), Some(RegStep::Referral));
    }

    #[test]
    fn test_previous_from_success_is_none() {
        assert_eq!(RegStep::Success.previous(), None);
    }

    #[test]
    fn test_step_numbers() {
        assert_eq!(RegStep::Referral.number(), 1);
        assert_eq!(RegStep::DefaultReferral.number(), 1);
        assert_eq!(RegStep::Register.number(), 2);
        assert_eq!(RegStep::Success.number(), 3);
    }

    #[test]
    fn test_announcements() {
        assert!(RegStep::Referral.announcement().contains("Referral"));
        assert!(RegStep::DefaultReferral.announcement().contains("Invitation"));
        assert!(RegStep::Register.announcement().contains("Registration"));
        assert!(RegStep::Success.announcement().contains("successful"));
    }

    #[test]
    fn test_element_ids() {
        assert_eq!(RegStep::Referral.element_id(), "referralStep");
        assert_eq!(RegStep::DefaultReferral.element_id(), "defaultReferralStep");
        assert_eq!(RegStep::Register.element_id(), "registrationStep");
        assert_eq!(RegStep::Success.element_id(), "successStep");
    }
}
