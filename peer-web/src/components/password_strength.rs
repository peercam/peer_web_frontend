//! Password strength meter component for the registration form.

use leptos::prelude::*;

use crate::components::validation::PasswordValidation;

/// Visual password strength indicator with requirement checklist.
///
/// Renders a 5-segment strength bar, a label (e.g. "Good"), and a list
/// of unmet requirements that hides items as they are satisfied.
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
    let strength_label = Memo::new(move |_| validation.get().strength.label());

    let label_class = Memo::new(move |_| validation.get().strength.label_class());

    // Derived: which requirements are still unmet
    let requirements = Memo::new(move |_| validation.get().requirements);

    // Derived: numeric strength value (1-5) for aria-valuenow
    let strength_numeric = Memo::new(move |_| validation.get().strength.numeric());

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
            <div
                class="strength-meter"
                role="meter"
                aria-label="Password strength"
                aria-valuenow=move || strength_numeric.get().to_string()
                aria-valuemin="1"
                aria-valuemax="5"
                aria-valuetext=move || {
                    format!("Password strength: {}", strength_label.get())
                }
            >
                <div class=move || fill_class.get() id="strengthFill">
                    <span class="strength-segment segment-weak" aria-hidden="true"></span>
                    <span class="strength-segment segment-weak2" aria-hidden="true"></span>
                    <span class="strength-segment segment-medium" aria-hidden="true"></span>
                    <span class="strength-segment segment-strong" aria-hidden="true"></span>
                    <span class="strength-segment segment-excellent" aria-hidden="true"></span>
                </div>
            </div>

            // Screen reader announcement for strength changes
            <div class="sr-only" aria-live="polite" aria-atomic="true">
                {move || {
                    if visible.get() {
                        format!("Password strength: {}", strength_label.get())
                    } else {
                        String::new()
                    }
                }}
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
