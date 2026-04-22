//! Client-side validation helpers for registration form fields.
//!
//! These run in the browser (WASM) for instant feedback.
//! Server-side validation in `src/api/validation.rs` (Step 4)
//! provides defense-in-depth.

/// Individual password requirement checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordRequirements {
    /// At least 8 characters
    pub length: bool,
    /// Contains at least one lowercase letter
    pub lowercase: bool,
    /// Contains at least one uppercase letter
    pub uppercase: bool,
    /// Contains at least one digit
    pub number: bool,
    /// Contains a special character OR is 12+ characters
    pub special: bool,
}

impl PasswordRequirements {
    /// Check if all required criteria are met.
    /// Required: length, lowercase, uppercase, number
    /// Optional (bonus): special
    pub fn is_sufficient(&self) -> bool {
        self.length && self.lowercase && self.uppercase && self.number
    }

    /// Count how many requirements are met (out of 5).
    pub fn met_count(&self) -> u8 {
        let mut count = 0;
        if self.length {
            count += 1;
        }
        if self.lowercase {
            count += 1;
        }
        if self.uppercase {
            count += 1;
        }
        if self.number {
            count += 1;
        }
        if self.special {
            count += 1;
        }
        count
    }
}

/// Password strength levels for the meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// 0–1 requirements met
    VeryWeak,
    /// 2 requirements met
    Weak,
    /// 3 requirements met
    NeedsImprovement,
    /// All 4 required criteria met
    Good,
    /// All 4 required + special character
    Excellent,
}

impl PasswordStrength {
    /// CSS class suffix for the strength meter.
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::VeryWeak => "weak",
            Self::Weak => "weak2",
            Self::NeedsImprovement => "medium",
            Self::Good => "strong",
            Self::Excellent => "excellent",
        }
    }

    /// Numeric value (1–5) for aria-valuenow.
    pub fn numeric(&self) -> u8 {
        match self {
            Self::VeryWeak => 1,
            Self::Weak => 2,
            Self::NeedsImprovement => 3,
            Self::Good => 4,
            Self::Excellent => 5,
        }
    }

    /// Human-readable label for screen readers and display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::VeryWeak => "Very weak",
            Self::Weak => "Weak",
            Self::NeedsImprovement => "Needs improvement",
            Self::Good => "Good",
            Self::Excellent => "Excellent",
        }
    }

    /// CSS class for the label text.
    pub fn label_class(&self) -> &'static str {
        match self {
            Self::VeryWeak => "very-weak",
            Self::Weak => "weak",
            Self::NeedsImprovement => "improvement",
            Self::Good => "good",
            Self::Excellent => "excellent",
        }
    }
}

/// Result of password validation with detailed requirements + strength.
#[derive(Debug, Clone, PartialEq)]
pub struct PasswordValidation {
    pub requirements: PasswordRequirements,
    pub strength: PasswordStrength,
}

/// Validate email format.
///
/// Regex equivalent: `/^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/`
pub fn is_valid_email(input: &str) -> bool {
    let input = input.trim();

    // Quick length check
    if input.len() < 5 || input.len() > 254 {
        return false;
    }

    // Must contain exactly one @
    let at_count = input.chars().filter(|&c| c == '@').count();
    if at_count != 1 {
        return false;
    }

    let parts: Vec<&str> = input.split('@').collect();
    let (local, domain) = (parts[0], parts[1]);

    // Local part: non-empty, alphanumeric + ._%+-
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    if !local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c))
    {
        return false;
    }

    // Domain part: must have at least one dot, valid chars
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }

    let domain_parts: Vec<&str> = domain.split('.').collect();

    // TLD must be at least 2 chars
    if let Some(tld) = domain_parts.last()
        && (tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()))
    {
        return false;
    }

    // All domain parts must be valid
    domain_parts
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
}

/// Validate username format.
///
/// Rules: 3–23 characters, alphanumeric + underscore + hyphen only.
pub fn is_valid_username(input: &str) -> bool {
    let input = input.trim();
    let len = input.len();

    if !(3..=23).contains(&len) {
        return false;
    }

    input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Validate a password and return detailed requirements + strength.
pub fn validate_password(password: &str) -> PasswordValidation {
    let requirements = PasswordRequirements {
        length: password.len() >= 8,
        lowercase: password.chars().any(|c| c.is_ascii_lowercase()),
        uppercase: password.chars().any(|c| c.is_ascii_uppercase()),
        number: password.chars().any(|c| c.is_ascii_digit()),
        special: password
            .chars()
            .any(|c| "!@#$%^&*(),.?\":{}|<>".contains(c))
            || password.len() >= 12,
    };

    let strength = if requirements.is_sufficient() {
        if requirements.special {
            PasswordStrength::Excellent
        } else {
            PasswordStrength::Good
        }
    } else {
        match requirements.met_count() {
            0 | 1 => PasswordStrength::VeryWeak,
            2 => PasswordStrength::Weak,
            _ => PasswordStrength::NeedsImprovement,
        }
    };

    PasswordValidation {
        requirements,
        strength,
    }
}

/// Check if confirm password matches the original password.
pub fn passwords_match(password: &str, confirm_password: &str) -> bool {
    !confirm_password.is_empty() && password == confirm_password
}

/// Validate that a string matches UUID format.
///
/// Accepts: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (hex digits, case-insensitive).
/// Does not enforce UUID v4 variant bits — the backend handles that.
pub fn is_valid_uuid(input: &str) -> bool {
    let input = input.trim();
    if input.len() != 36 {
        return false;
    }

    input.chars().enumerate().all(|(i, c)| match i {
        8 | 13 | 18 | 23 => c == '-',
        _ => c.is_ascii_hexdigit(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_uuid_lowercase() {
        assert!(is_valid_uuid("85d5f836-b1f5-4c4e-9381-1b058e13df93"));
    }

    #[test]
    fn valid_uuid_uppercase() {
        assert!(is_valid_uuid("85D5F836-B1F5-4C4E-9381-1B058E13DF93"));
    }

    #[test]
    fn valid_uuid_mixed_case() {
        assert!(is_valid_uuid("85d5F836-b1F5-4c4E-9381-1b058E13df93"));
    }

    #[test]
    fn valid_uuid_with_whitespace_trimmed() {
        assert!(is_valid_uuid("  85d5f836-b1f5-4c4e-9381-1b058e13df93  "));
    }

    #[test]
    fn invalid_empty_string() {
        assert!(!is_valid_uuid(""));
    }

    #[test]
    fn invalid_short_string() {
        assert!(!is_valid_uuid("abc123"));
    }

    #[test]
    fn invalid_no_hyphens() {
        assert!(!is_valid_uuid("85d5f836b1f54c4e93811b058e13df93"));
    }

    #[test]
    fn invalid_wrong_hyphen_positions() {
        assert!(!is_valid_uuid("85d5f83-6b1f5-4c4e-9381-1b058e13df93"));
    }

    #[test]
    fn invalid_non_hex_characters() {
        assert!(!is_valid_uuid("85d5f836-b1f5-4c4e-9381-1b058e13dg93"));
    }

    #[test]
    fn invalid_too_long() {
        assert!(!is_valid_uuid("85d5f836-b1f5-4c4e-9381-1b058e13df930"));
    }

    // --- Email tests ---

    #[test]
    fn valid_email_simple() {
        assert!(is_valid_email("user@example.com"));
    }

    #[test]
    fn valid_email_with_plus() {
        assert!(is_valid_email("user+tag@example.com"));
    }

    #[test]
    fn valid_email_subdomain() {
        assert!(is_valid_email("user@mail.sub.example.org"));
    }

    #[test]
    fn invalid_email_no_at() {
        assert!(!is_valid_email("userexample.com"));
    }

    #[test]
    fn invalid_email_no_domain() {
        assert!(!is_valid_email("user@"));
    }

    #[test]
    fn invalid_email_no_tld() {
        assert!(!is_valid_email("user@example"));
    }

    #[test]
    fn invalid_email_short_tld() {
        assert!(!is_valid_email("user@example.x"));
    }

    // --- Username tests ---

    #[test]
    fn valid_username_min_length() {
        assert!(is_valid_username("abc"));
    }

    #[test]
    fn valid_username_max_length() {
        assert!(is_valid_username("abcdefghij12345678901ab")); // 23 chars
    }

    #[test]
    fn valid_username_with_underscore() {
        assert!(is_valid_username("peer_user"));
    }

    #[test]
    fn valid_username_with_hyphen() {
        assert!(is_valid_username("peer-user"));
    }

    #[test]
    fn invalid_username_too_short() {
        assert!(!is_valid_username("ab"));
    }

    #[test]
    fn invalid_username_too_long() {
        assert!(!is_valid_username("abcdefghij123456789012ab")); // 24 chars
    }

    #[test]
    fn invalid_username_special_chars() {
        assert!(!is_valid_username("user@name"));
        assert!(!is_valid_username("user.name"));
        assert!(!is_valid_username("user name"));
    }

    // --- Password tests ---

    #[test]
    fn password_very_weak() {
        let result = validate_password("abc");
        assert_eq!(result.strength, PasswordStrength::VeryWeak);
        assert!(!result.requirements.is_sufficient());
    }

    #[test]
    fn password_weak() {
        let result = validate_password("abcdefgh");
        assert_eq!(result.strength, PasswordStrength::Weak);
    }

    #[test]
    fn password_needs_improvement() {
        let result = validate_password("Abcdefgh");
        assert_eq!(result.strength, PasswordStrength::NeedsImprovement);
    }

    #[test]
    fn password_good() {
        let result = validate_password("Abcd1234");
        assert_eq!(result.strength, PasswordStrength::Good);
        assert!(result.requirements.is_sufficient());
    }

    #[test]
    fn password_excellent_with_special() {
        let result = validate_password("Abcd1234!");
        assert_eq!(result.strength, PasswordStrength::Excellent);
    }

    #[test]
    fn password_excellent_with_length() {
        let result = validate_password("Abcdefgh1234"); // 12 chars, no special
        assert_eq!(result.strength, PasswordStrength::Excellent);
    }

    // --- Confirm password tests ---

    #[test]
    fn passwords_match_valid() {
        assert!(passwords_match("Abcd1234", "Abcd1234"));
    }

    #[test]
    fn passwords_match_empty_confirm() {
        assert!(!passwords_match("Abcd1234", ""));
    }

    #[test]
    fn passwords_match_different() {
        assert!(!passwords_match("Abcd1234", "Abcd12345"));
    }
}
