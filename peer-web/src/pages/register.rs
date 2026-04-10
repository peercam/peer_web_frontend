//! Registration page component.
//!
//! This is a multi-step registration flow:
//! 1. Referral code entry and verification
//! 2. Registration form (email, username, password)
//! 3. Success confirmation

use leptos::prelude::*;

/// The registration page component.
///
/// This is a placeholder that will be expanded in Steps 6-10.
#[component]
pub fn RegisterPage() -> impl IntoView {
    view! {
        <div class="container">
            <div class="form-box">
                <h1>"Create Account"</h1>
                <p>"Registration form coming in Step 6..."</p>
            </div>
        </div>
    }
}
