//! Server-side input validation for registration server functions.
//!
//! These checks run on the Axum server before any GraphQL call is made.
//! They provide fast rejection of obviously invalid input and protect
//! against malformed requests that bypass client-side validation.

use crate::models::common::ApiError;

/// Validate that a string looks like a UUID v4.
///
/// Accepts the format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
/// where `x` is a hex digit. Does not enforce v4 variant bits.
pub fn validate_uuid(s: &str) -> Result<(), ApiError> {
    let uuid_pattern = regex::Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
    )
    .expect("UUID regex is valid");

    if uuid_pattern.is_match(s) {
        Ok(())
    } else {
        Err(ApiError::Api {
            code: "31010".to_string(),
            message: "Invalid referral code format".to_string(),
        })
    }
}

/// Validate registration input fields on the server side.
///
/// Checks:
/// - Email is non-empty and has a basic valid format
/// - Username is 3–23 characters, alphanumeric with `_` and `-`
/// - Password is at least 8 characters
///
/// These mirror the client-side checks but are intentionally more lenient —
/// the real validation authority is the backend GraphQL API.
pub fn validate_registration_input(
    email: &str,
    username: &str,
    password: &str,
) -> Result<(), ApiError> {
    // Email: basic format check
    if email.is_empty() || !email.contains('@') || !email.contains('.') {
        return Err(ApiError::Api {
            code: "30103".to_string(),
            message: "Invalid email format".to_string(),
        });
    }

    // Email: length sanity check (RFC 5321 limit is 254)
    if email.len() > 254 {
        return Err(ApiError::Api {
            code: "30103".to_string(),
            message: "Email address too long".to_string(),
        });
    }

    // Username: 3-23 chars, alphanumeric + underscore + hyphen
    let username_pattern =
        regex::Regex::new(r"^[a-zA-Z0-9_-]{3,23}$").expect("Username regex is valid");

    if !username_pattern.is_match(username) {
        return Err(ApiError::Api {
            code: "30202".to_string(),
            message: "Invalid username format".to_string(),
        });
    }

    // Password: minimum 8 characters
    if password.len() < 8 {
        return Err(ApiError::Api {
            code: "30103".to_string(),
            message: "Password must be at least 8 characters".to_string(),
        });
    }

    Ok(())
}

/// Validate a username.
///
/// Checks: 3-23 chars, alphanumeric with `_` and `-`
pub fn validate_username(username: &str) -> Result<(), ApiError> {
    let username_pattern =
        regex::Regex::new(r"^[a-zA-Z0-9_-]{3,23}$").expect("Username regex is valid");

    if !username_pattern.is_match(username) {
        return Err(ApiError::Api {
            code: "30202".to_string(),
            message: "Invalid username format".to_string(),
        });
    }

    Ok(())
}

/// Validate a password.
///
/// Checks: minimum 8 characters
pub fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::Api {
            code: "30103".to_string(),
            message: "Password must be at least 8 characters".to_string(),
        });
    }

    Ok(())
}

/// Validate an email address.
///
/// Checks: non-empty, contains @ and ., reasonable length
pub fn validate_email(email: &str) -> Result<(), ApiError> {
    if email.is_empty() || !email.contains('@') || !email.contains('.') {
        return Err(ApiError::Api {
            code: "30103".to_string(),
            message: "Invalid email format".to_string(),
        });
    }

    // Email: length sanity check (RFC 5321 limit is 254)
    if email.len() > 254 {
        return Err(ApiError::Api {
            code: "30103".to_string(),
            message: "Email address too long".to_string(),
        });
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- UUID validation ---

    #[test]
    fn test_valid_uuid() {
        assert!(validate_uuid("85d5f836-b1f5-4c4e-9381-1b058e13df93").is_ok());
        assert!(validate_uuid("00000000-0000-0000-0000-000000000000").is_ok());
        assert!(validate_uuid("ABCDEF01-2345-6789-ABCD-EF0123456789").is_ok());
    }

    #[test]
    fn test_invalid_uuid() {
        assert!(validate_uuid("").is_err());
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("85d5f836b1f54c4e93811b058e13df93").is_err()); // No dashes
        assert!(validate_uuid("85d5f836-b1f5-4c4e-9381").is_err()); // Too short
        assert!(validate_uuid("85d5f836-b1f5-4c4e-9381-1b058e13df93-extra").is_err());
    }

    // --- Registration input validation ---

    #[test]
    fn test_valid_registration_input() {
        assert!(validate_registration_input("user@example.com", "peer_user", "SecurePass123!",).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(validate_registration_input("", "user", "Pass1234").is_err());
        assert!(validate_registration_input("noatsign", "user", "Pass1234").is_err());
        assert!(validate_registration_input("no@dot", "user", "Pass1234").is_err());
    }

    #[test]
    fn test_invalid_username() {
        assert!(validate_registration_input("a@b.c", "ab", "Pass1234").is_err()); // Too short
        assert!(validate_registration_input("a@b.c", &"a".repeat(24), "Pass1234").is_err()); // Too long
        assert!(validate_registration_input("a@b.c", "user name", "Pass1234").is_err()); // Space
        assert!(validate_registration_input("a@b.c", "user@name", "Pass1234").is_err()); // @ symbol
    }

    #[test]
    fn test_invalid_password() {
        assert!(validate_registration_input("a@b.c", "user", "short").is_err()); // < 8 chars
        assert!(validate_registration_input("a@b.c", "user", "1234567").is_err()); // 7 chars
    }

    #[test]
    fn test_boundary_username_lengths() {
        // Exactly 3 chars — valid
        assert!(validate_registration_input("a@b.c", "abc", "Pass1234").is_ok());
        // Exactly 23 chars — valid
        assert!(validate_registration_input("a@b.c", &"a".repeat(23), "Pass1234").is_ok());
    }
}
