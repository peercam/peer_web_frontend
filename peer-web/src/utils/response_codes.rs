//! Maps backend response codes to user-friendly messages.

use std::collections::HashMap;
use once_cell::sync::Lazy;

static RESPONSE_MESSAGES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Referral verification codes
    m.insert("11011", "Referral information loaded.");
    m.insert("31010", "The referral code you entered is invalid. Please check and try again.");
    m.insert("41013", "We're having trouble loading your referral information. Please try again.");

    // Registration codes
    m.insert("10601", "Registration successful! Please check your email to verify your account.");
    m.insert("30601", "This email is already registered. Please log in or use a different email.");
    m.insert("40601", "Registration failed due to a server error. Please try again.");
    m.insert("40602", "Registration failed: unable to generate user ID. Please try again.");

    // Account verification codes
    m.insert("10701", "Account verified! You can now log in.");
    m.insert("30701", "This account is already verified.");
    m.insert("40701", "Verification failed due to a server error. Please try again.");

    m
});

const DEFAULT_MESSAGE: &str = "An unexpected error occurred. Please try again.";

/// Returns a user-friendly message for the given response code.
pub fn user_friendly_msg(code: &str) -> &'static str {
    RESPONSE_MESSAGES.get(code).copied().unwrap_or(DEFAULT_MESSAGE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_codes() {
        assert_eq!(user_friendly_msg("11011"), "Referral information loaded.");
        assert_eq!(
            user_friendly_msg("31010"),
            "The referral code you entered is invalid. Please check and try again."
        );
        assert_eq!(
            user_friendly_msg("10601"),
            "Registration successful! Please check your email to verify your account."
        );
    }

    #[test]
    fn test_unknown_code_returns_default() {
        assert_eq!(user_friendly_msg("99999"), DEFAULT_MESSAGE);
        assert_eq!(user_friendly_msg(""), DEFAULT_MESSAGE);
    }
}
