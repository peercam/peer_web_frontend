//! Maps backend response codes to user-friendly messages.
//!
//! Based on `json/response-codes.json`. Only codes that can appear
//! during the registration flow are included here; additional codes
//! can be added as more pages are migrated.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Compiled map of response code → user-friendly message.
static RESPONSE_CODES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // ── Registration & Account ───────────────────────────────────────
    m.insert("10601", "Registration successful! Please check your email to verify your account.");
    m.insert("10701", "Account verified! You can now log in.");
    m.insert("10801", "Login successful! Welcome back.");

    // ── Referral ─────────────────────────────────────────────────────
    m.insert("11011", "Referral information loaded.");
    m.insert("21002", "Creating new referral link for you.");
    m.insert("21003", "No referral information available.");
    m.insert("31010", "The referral code you entered is invalid. Please check and try again.");
    m.insert("41013", "We're having trouble loading your referral information. Please try again.");

    // ── Validation (30xxx) ───────────────────────────────────────────
    m.insert("30101", "Please fill in all required fields.");
    m.insert("30102", "Some fields are empty. Please complete them.");
    m.insert("30103", "Some fields have invalid format. Please correct them.");
    m.insert("30202", "Username must be 3-23 chars with letters, numbers, or underscores.");
    m.insert("30224", "Please enter a valid email address.");
    m.insert("30226", "Password must be 8-128 chars with uppercase, lowercase, numbers and special characters.");
    m.insert("30231", "Please fill in all required fields correctly.");
    m.insert("30301", "Missing required fields. Please fill in all fields.");

    // ── Duplicate / Conflict (306xx) ─────────────────────────────────
    m.insert("30601", "Email already registered. Use a different one.");
    m.insert("30701", "Account already verified!");

    // ── Invalid Action (31xxx) ───────────────────────────────────────
    m.insert("30801", "Invalid email or password. Please try again.");
    m.insert("31007", "Invalid referral code. Please check and try again.");

    // ── Server Errors (4xxxx) ────────────────────────────────────────
    m.insert("40601", "We couldn't complete your registration. Please try again or contact support.");
    m.insert("40602", "We're having trouble creating your account. Please try again.");
    m.insert("40701", "We couldn't verify your account. Please try again or contact support.");

    m
});

/// Look up the user-friendly message for a response code.
///
/// Returns the mapped message if the code is known, otherwise
/// returns the code itself as a fallback (matching the JS
/// `userfriendlymsg()` behaviour).
pub fn user_friendly_msg(code: &str) -> &str {
    RESPONSE_CODES.get(code).copied().unwrap_or(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("10601"),
            "Registration successful! Please check your email to verify your account."
        );
    }

    #[test]
    fn duplicate_email_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("30601"),
            "Email already registered. Use a different one."
        );
    }

    #[test]
    fn invalid_referral_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("31010"),
            "The referral code you entered is invalid. Please check and try again."
        );
    }

    #[test]
    fn server_error_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("40601"),
            "We couldn't complete your registration. Please try again or contact support."
        );
    }

    #[test]
    fn unknown_code_returns_code_itself() {
        assert_eq!(user_friendly_msg("99999"), "99999");
    }

    #[test]
    fn empty_code_returns_empty_string() {
        assert_eq!(user_friendly_msg(""), "");
    }
}
