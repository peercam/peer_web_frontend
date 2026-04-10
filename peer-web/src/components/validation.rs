//! Client-side validation helpers for registration form fields.
//!
//! These run in the browser (WASM) for instant feedback.
//! Server-side validation in `src/api/validation.rs` (Step 4)
//! provides defense-in-depth.

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
        assert!(is_valid_uuid(
            "  85d5f836-b1f5-4c4e-9381-1b058e13df93  "
        ));
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
        assert!(!is_valid_uuid(
            "85d5f836-b1f5-4c4e-9381-1b058e13df930"
        ));
    }
}
