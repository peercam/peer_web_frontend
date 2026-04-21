//! Pure submit-gate logic for the registration form (Step 2).
//!
//! This module deliberately contains **no Leptos reactive types** and
//! **no DOM access**. Every decision the form makes about whether the
//! "Create Account" button can be submitted, and which checkbox-error
//! message to show, lives here as plain Rust functions.
//!
//! ## Why a separate module
//!
//! The Playwright tests `form-validation.spec.ts` T5/T6/T7/T7b assert
//! UI-level behaviour ("button is disabled until requirements are met",
//! "error message says 'do not match'", etc.). That behaviour is
//! ultimately a function of six inputs (email, username, password,
//! confirm, privacy, eula) and one output (can-submit + which message).
//! By making that function pure we can cover every branch with cheap
//! `cargo test --lib` unit tests and reserve the browser for things
//! only the browser can answer (focus order, computed CSS, axe).
//!
//! ## Mapping back to E2E coverage
//!
//! | E2E test                           | Covered by                         |
//! |------------------------------------|------------------------------------|
//! | T5  weak password rejected         | `can_submit_registration` weak-pwd |
//! | T6  mismatched confirm password    | `can_submit_registration` mismatch |
//! | T7  both checkboxes unchecked      | `validate_checkboxes::BothRequired`|
//! | T7b single checkbox unchecked      | `validate_checkboxes::EulaRequired`|

use crate::components::validation::{
    is_valid_email, is_valid_username, passwords_match, validate_password,
};

/// Snapshot of every input that participates in the submit decision.
///
/// Borrowing rather than owning keeps callers from having to clone
/// signal-derived strings just to ask "can I submit?".
#[derive(Debug, Clone, Copy)]
pub struct RegistrationFields<'a> {
    pub email: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub confirm_password: &'a str,
    pub privacy_accepted: bool,
    pub eula_accepted: bool,
}

/// Returns `true` iff every field is valid AND both checkboxes are
/// checked. Matches the predicate the `RegistrationStep` component
/// uses to enable/disable the submit button.
pub fn can_submit_registration(f: RegistrationFields<'_>) -> bool {
    is_valid_email(f.email)
        && is_valid_username(f.username)
        && validate_password(f.password).requirements.is_sufficient()
        && passwords_match(f.password, f.confirm_password)
        && f.privacy_accepted
        && f.eula_accepted
}

/// Result of inspecting just the legal-checkbox state.
///
/// Returned by [`validate_checkboxes`] and consumed by the form's
/// submit handler to decide which message — if any — to surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxError {
    /// Both Privacy Policy and EULA are checked.
    None,
    /// Neither checkbox is checked.
    BothRequired,
    /// Only the Privacy Policy is unchecked.
    PrivacyRequired,
    /// Only the EULA is unchecked.
    EulaRequired,
}

impl CheckboxError {
    /// User-visible message text. Empty string for [`Self::None`] so
    /// callers can bind it directly to an `aria-live` region.
    pub fn message(self) -> &'static str {
        match self {
            Self::None => "",
            Self::BothRequired => "Please accept both the Privacy Policy and EULA",
            Self::PrivacyRequired => "Please accept the Privacy Policy to continue",
            Self::EulaRequired => {
                "Please accept the End User License Agreement (EULA) to continue"
            }
        }
    }

    /// `true` when an error message should be displayed.
    pub fn has_error(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Inspect the two legal-acceptance checkboxes and report which (if any)
/// require the user's attention. The order of checks mirrors the
/// branching in `RegistrationStep::handle_submit`.
pub fn validate_checkboxes(privacy: bool, eula: bool) -> CheckboxError {
    match (privacy, eula) {
        (true, true) => CheckboxError::None,
        (false, false) => CheckboxError::BothRequired,
        (false, true) => CheckboxError::PrivacyRequired,
        (true, false) => CheckboxError::EulaRequired,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A baseline of fields that *would* pass every check. Individual
    // tests clone and tweak one field to assert the gate flips off.
    fn ok() -> RegistrationFields<'static> {
        RegistrationFields {
            email: "happy@example.com",
            username: "happy_user",
            password: "StrongPass99!",
            confirm_password: "StrongPass99!",
            privacy_accepted: true,
            eula_accepted: true,
        }
    }

    // ── can_submit_registration ────────────────────────────────────

    #[test]
    fn baseline_can_submit() {
        assert!(can_submit_registration(ok()));
    }

    /// E2E `form-validation.spec.ts T5`: weak password keeps the
    /// "Create Account" button disabled.
    #[test]
    fn t5_weak_password_blocks_submit() {
        let mut f = ok();
        f.password = "abcd";
        f.confirm_password = "abcd";
        assert!(!can_submit_registration(f));
    }

    #[test]
    fn t5_password_missing_uppercase_blocks_submit() {
        let mut f = ok();
        f.password = "lowercase1!";
        f.confirm_password = "lowercase1!";
        assert!(!can_submit_registration(f));
    }

    #[test]
    fn t5_password_missing_digit_blocks_submit() {
        let mut f = ok();
        f.password = "NoDigitsHere!";
        f.confirm_password = "NoDigitsHere!";
        // 12+ chars satisfies "special" via the length-bonus rule, but
        // still requires a digit — "NoDigitsHere!" lacks one.
        assert!(!can_submit_registration(f));
    }

    /// E2E `form-validation.spec.ts T6`: confirm-password mismatch
    /// keeps the submit button disabled.
    #[test]
    fn t6_mismatched_confirm_blocks_submit() {
        let mut f = ok();
        f.confirm_password = "DifferentPass456!";
        assert!(!can_submit_registration(f));
    }

    #[test]
    fn t6_empty_confirm_blocks_submit() {
        let mut f = ok();
        f.confirm_password = "";
        assert!(!can_submit_registration(f));
    }

    /// E2E `form-validation.spec.ts T7`: both checkboxes unchecked.
    #[test]
    fn t7_both_checkboxes_unchecked_blocks_submit() {
        let mut f = ok();
        f.privacy_accepted = false;
        f.eula_accepted = false;
        assert!(!can_submit_registration(f));
    }

    /// E2E `form-validation.spec.ts T7b`: single checkbox unchecked.
    #[test]
    fn t7b_eula_unchecked_blocks_submit() {
        let mut f = ok();
        f.eula_accepted = false;
        assert!(!can_submit_registration(f));
    }

    #[test]
    fn t7b_privacy_unchecked_blocks_submit() {
        let mut f = ok();
        f.privacy_accepted = false;
        assert!(!can_submit_registration(f));
    }

    #[test]
    fn invalid_email_blocks_submit() {
        let mut f = ok();
        f.email = "not-an-email";
        assert!(!can_submit_registration(f));
    }

    #[test]
    fn invalid_username_blocks_submit() {
        let mut f = ok();
        f.username = "ab"; // too short
        assert!(!can_submit_registration(f));
    }

    // ── validate_checkboxes ────────────────────────────────────────

    #[test]
    fn checkboxes_both_checked_returns_none() {
        let result = validate_checkboxes(true, true);
        assert_eq!(result, CheckboxError::None);
        assert!(!result.has_error());
        assert_eq!(result.message(), "");
    }

    #[test]
    fn checkboxes_neither_checked_returns_both_required() {
        let result = validate_checkboxes(false, false);
        assert_eq!(result, CheckboxError::BothRequired);
        assert!(result.has_error());
        assert!(result.message().contains("both"));
    }

    #[test]
    fn checkboxes_only_privacy_unchecked() {
        let result = validate_checkboxes(false, true);
        assert_eq!(result, CheckboxError::PrivacyRequired);
        assert!(result.message().contains("Privacy Policy"));
    }

    #[test]
    fn checkboxes_only_eula_unchecked() {
        let result = validate_checkboxes(true, false);
        assert_eq!(result, CheckboxError::EulaRequired);
        assert!(result.message().contains("EULA"));
    }
}
