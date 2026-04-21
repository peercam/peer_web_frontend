//! JWT token utilities.
//!
//! Provides helpers for decoding JWT claims without full verification
//! (verification happens server-side).

use serde::Deserialize;

/// JWT claims we care about (expiry).
#[derive(Debug, Deserialize)]
struct JwtClaims {
    /// Expiry timestamp (Unix seconds).
    exp: i64,
}

/// Decode a JWT and extract the expiry timestamp.
///
/// This performs base64 decoding of the payload only (no signature verification).
/// Returns `None` if the token is malformed.
pub fn token_expiry_secs(token: &str) -> Option<i64> {
    // JWT format: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decode the payload (middle part)
    let payload = parts[1];

    // JWT uses base64url encoding (no padding)
    let decoded = base64_url_decode(payload)?;
    let json_str = std::str::from_utf8(&decoded).ok()?;

    let claims: JwtClaims = serde_json::from_str(json_str).ok()?;
    Some(claims.exp)
}

/// Base64url decode without padding.
fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    // Add padding if needed
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };

    // Convert base64url to standard base64
    let standard: String = padded
        .chars()
        .map(|c| match c {
            '-' => '+',
            '_' => '/',
            c => c,
        })
        .collect();

    // Use web-sys for WASM
    #[cfg(feature = "hydrate")]
    {
        let window = leptos::web_sys::window()?;
        let result = window.atob(&standard).ok()?;
        Some(result.into_bytes())
    }

    // Server-side: manual base64 decoding (avoid external dependency)
    #[cfg(not(feature = "hydrate"))]
    {
        // Standard Base64 alphabet
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        fn decode_char(c: u8) -> Option<u8> {
            ALPHABET.iter().position(|&x| x == c).map(|p| p as u8)
        }

        let bytes = standard.as_bytes();
        let mut output = Vec::with_capacity(bytes.len() * 3 / 4);

        for chunk in bytes.chunks(4) {
            if chunk.len() < 4 {
                break;
            }

            let a = decode_char(chunk[0])?;
            let b = decode_char(chunk[1])?;

            output.push((a << 2) | (b >> 4));

            if chunk[2] != b'=' {
                let c = decode_char(chunk[2])?;
                output.push((b << 4) | (c >> 2));

                if chunk[3] != b'=' {
                    let d = decode_char(chunk[3])?;
                    output.push((c << 6) | d);
                }
            }
        }

        Some(output)
    }
}

/// Calculate seconds until token expires.
///
/// Returns `None` if token is invalid or already expired.
pub fn seconds_until_expiry(token: &str) -> Option<i64> {
    let exp = token_expiry_secs(token)?;

    #[cfg(feature = "hydrate")]
    {
        let now = (js_sys::Date::now() / 1000.0) as i64;
        let remaining = exp - now;
        if remaining > 0 { Some(remaining) } else { None }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
        let remaining = exp - now;
        if remaining > 0 { Some(remaining) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_expiry_extraction() {
        // Example JWT with exp claim (you'd use a real test token in practice)
        // This is a minimal test - real tokens would have actual signatures
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MTI3NTIwMDB9.signature";
        let exp = token_expiry_secs(token);
        assert_eq!(exp, Some(1712752000));
    }

    #[test]
    fn test_invalid_token() {
        assert_eq!(token_expiry_secs("invalid"), None);
        assert_eq!(token_expiry_secs("a.b"), None);
        assert_eq!(token_expiry_secs(""), None);
    }
}
