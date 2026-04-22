//! Client-side cookie utilities for WASM.
//!
//! These are used for reading non-HttpOnly cookies (like `remember_me`)
//! in the browser. Auth tokens are HttpOnly and managed server-side.

/// Get a cookie value by name (client-side only).
pub fn get_cookie(name: &str) -> Option<String> {
    #[cfg(feature = "hydrate")]
    {
        // Use helper to get document.cookie via HtmlDocument cast
        let cookies = get_document_cookie()?;
        cookies.split(';').map(|s| s.trim()).find_map(|cookie| {
            let mut parts = cookie.splitn(2, '=');
            let key = parts.next()?.trim();
            let val = parts.next()?.trim();
            if key == name {
                Some(val.to_string())
            } else {
                None
            }
        })
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = name;
        None
    }
}

/// Get document.cookie using JavaScript
#[cfg(feature = "hydrate")]
fn get_document_cookie() -> Option<String> {
    use wasm_bindgen::JsCast;
    let window = leptos::web_sys::window()?;
    let document = window.document()?;
    // Cast to HtmlDocument to access cookie property
    let html_doc = document.dyn_into::<leptos::web_sys::HtmlDocument>().ok()?;
    html_doc.cookie().ok()
}

/// Set document.cookie using JavaScript
#[cfg(feature = "hydrate")]
fn set_document_cookie(cookie: &str) {
    use wasm_bindgen::JsCast;
    if let Some(window) = leptos::web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(html_doc) = document.dyn_into::<leptos::web_sys::HtmlDocument>() {
                let _ = html_doc.set_cookie(cookie);
            }
        }
    }
}

/// Set a cookie with optional max-age in days.
pub fn set_cookie(name: &str, value: &str, days: Option<i32>) {
    #[cfg(feature = "hydrate")]
    {
        let max_age = days
            .map(|d| format!("; Max-Age={}", d * 24 * 60 * 60))
            .unwrap_or_default();
        let cookie = format!("{}={}{}; Path=/; SameSite=Strict", name, value, max_age);
        set_document_cookie(&cookie);
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (name, value, days);
    }
}

/// Delete a cookie by setting its Max-Age to 0.
pub fn delete_cookie(name: &str) {
    #[cfg(feature = "hydrate")]
    {
        let cookie = format!("{}=; Path=/; SameSite=Strict; Max-Age=0", name);
        set_document_cookie(&cookie);
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = name;
    }
}

/// Set the "remember me" preference cookie.
pub fn set_remember_me(value: bool) {
    if value {
        set_cookie("remember_me", "true", Some(30));
    } else {
        delete_cookie("remember_me");
    }
}

/// Check if "remember me" is set.
pub fn get_remember_me() -> bool {
    get_cookie("remember_me").is_some_and(|v| v == "true")
}
