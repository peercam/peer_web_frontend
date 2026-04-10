//! Referral code entry components for registration Step 1 / Step 1b.

use leptos::prelude::*;

use crate::components::validation::is_valid_uuid;

/// The default referral code shown when the user clicks "Don't have a code?"
/// Matches the value hardcoded in the current PHP version.
const DEFAULT_REFERRAL_CODE: &str = "85d5f836-b1f5-4c4e-9381-1b058e13df93";

/// Step 1: Referral code input form with client-side UUID validation.
///
/// The parent (`RegisterPage`) owns the `referral_code` signal so it
/// persists when navigating back from step 2.
#[component]
pub fn ReferralStep(
    /// The referral code signal, owned by the parent.
    referral_code: RwSignal<String>,
    /// Callback when the user clicks "Verify Code" with a valid UUID.
    on_verify: Action<String, ()>,
    /// Callback when the user clicks "Don't have a code?"
    on_show_default: Callback<()>,
) -> impl IntoView {
    // Derived: is the current input a valid UUID?
    let is_valid = Memo::new(move |_| is_valid_uuid(&referral_code.get()));

    // Derived: validation message text
    let validation_message = Memo::new(move |_| {
        let code = referral_code.get();
        if code.is_empty() {
            String::new()
        } else if is_valid.get() {
            String::new()
        } else {
            "Hmm\u{2026} that referral code doesn't seem to work. \
             Ask your friend to send you a new link, or use a Peer code."
                .to_string()
        }
    });

    // Derived: CSS class for the input field wrapper
    let field_class = Memo::new(move |_| {
        let code = referral_code.get();
        if code.is_empty() {
            "input-field"
        } else if is_valid.get() {
            "input-field valid"
        } else {
            "input-field invalid"
        }
    });

    view! {
        <form
            on:submit=move |ev| {
                ev.prevent_default();
                if is_valid.get() {
                    on_verify.dispatch(referral_code.get_untracked());
                }
            }
            novalidate=true
        >
            <div class="input-group">
                <div class=move || field_class.get() id="referralCodeField">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-referral"></i>
                    </span>
                    <input
                        type="text"
                        id="referralCode"
                        name="referralCode"
                        placeholder="Enter your referral code"
                        required=true
                        aria-describedby="referralCodeValidation referralHelp"
                        autocomplete="off"
                        prop:value=move || referral_code.get()
                        on:input=move |ev| {
                            referral_code.set(event_target_value(&ev));
                        }
                    />

                    // Validation icon: green check, shown only when valid
                    <span
                        class=move || {
                            if is_valid.get() {
                                "validation-icon show"
                            } else {
                                "validation-icon"
                            }
                        }
                        id="referralCodeValidIcon"
                        aria-hidden="true"
                    >
                        <i class="peer-icon peer-icon-tick-circle"></i>
                    </span>
                </div>

                // Validation message (reactive)
                <div
                    class="validation-message medium_font"
                    id="referralCodeValidation"
                    role="alert"
                    aria-live="polite"
                >
                    {move || validation_message.get()}
                </div>

                <div id="referralHelp" class="sr-only">
                    "Enter the referral code provided to you"
                </div>
            </div>

            <button
                type="submit"
                class="btn btn-primary"
                id="verifyReferralBtn"
                disabled=move || !is_valid.get()
            >
                "Verify Code"
            </button>
        </form>

        <div class="step-footer medium_font">
            <p>
                "Don\u{2019}t have a code? "
                <a
                    href="#"
                    on:click=move |ev| {
                        ev.prevent_default();
                        on_show_default.run(());
                    }
                >
                    "Click here"
                </a>
                " to get peer code"
            </p>
        </div>
    }
}

/// Step 1b: Displays the default referral code for users who don't have one.
#[component]
pub fn DefaultReferralView(
    /// Callback when user clicks "Use This Code" — passes the default UUID back.
    on_use_code: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="input-group">
            <div class="referral-code-display medium_font">
                <span class="input-icon" aria-hidden="true">
                    <i class="peer-icon peer-icon-referral"></i>
                </span>
                <span id="defaultReferralCode">{DEFAULT_REFERRAL_CODE}</span>
            </div>
        </div>

        <button
            type="button"
            class="btn btn-primary"
            id="useThisCodeBtn"
            on:click=move |_| {
                on_use_code.run(DEFAULT_REFERRAL_CODE.to_string());
            }
        >
            "Use This Code"
        </button>
    }
}
