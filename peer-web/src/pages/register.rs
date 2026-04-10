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

    // ── Read ?ref= query parameter on mount ─────────────────────────────
    let query = use_query_map();
    Effect::new(move |_| {
        if let Some(ref_code) = query.get().get("ref") {
            if !ref_code.is_empty() {
                referral_code.set(ref_code);
            }
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

    // ── Back-button handler ─────────────────────────────────────────────
    let on_back = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        let step = current_step.get();
        match step.previous() {
            Some(prev) => current_step.set(prev),
            None => {
                #[cfg(feature = "hydrate")]
                {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/login");
                    }
                }
            }
        }
    };

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
                        <Show when=move || current_step.get().show_back_button()>
                            <a
                                class="btn btn-secondary back-btn"
                                href="#"
                                on:click=on_back
                            >
                                <span aria-hidden="true">
                                    <i class="peer-icon medium_font peer-icon-arrow-left"></i>
                                </span>
                                "Back"
                            </a>
                        </Show>
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
                            // Referral form placeholder — Step 6 will replace this
                            <p class="medium_font">"[Referral code form — Step 6]"</p>
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
                            // Default referral display placeholder — Step 6 will replace this
                            <p class="medium_font">"[Default referral code — Step 6]"</p>
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
                            // Registration form placeholder — Step 8 will replace this
                            <p class="medium_font">"[Registration form — Step 8]"</p>
                        </div>

                        // Step 3: Success
                        <div
                            class=step_class(RegStep::Success)
                            data-step="3"
                            id="successStep"
                        >
                            <div class="success-message">
                                <div class="step-header">
                                    <span class="icon" aria-hidden="true">
                                        <i class="peer-icon peer-icon-good-tick-circle"></i>
                                    </span>
                                    <h2 class="x_large_font">
                                        "Welcome to " <strong>"peer!"</strong>
                                    </h2>
                                    <p class="large_font">
                                        "Your account is ready! Start exploring and earn your first token today."
                                    </p>
                                </div>
                                // Login link placeholder — Step 10 will replace this
                                <p class="medium_font">"[Continue to Login — Step 10]"</p>
                            </div>
                        </div>
                    </div>

                    // ── Footer area ─────────────────────────────────────
                    <div class="footer_area medium_font">
                        <p class="version version-number"></p>
                    </div>
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
}
