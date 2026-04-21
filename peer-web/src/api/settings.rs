//! Settings API server functions.
//!
//! Provides server functions for updating user profile settings,
//! account credentials, content preferences, and account lifecycle.

use leptos::prelude::*;

/// Update the user's profile image.
///
/// # Arguments
///
/// * `img` - Base64-encoded image data (data:image/*;base64,...)
///
/// # Response Codes
///
/// - `11004`: Profile picture updated successfully
/// - `30101`: Missing fields
/// - `60501`: Not authenticated
#[server(UpdateProfileImage, "/api")]
pub async fn update_profile_image(img: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{UPDATE_PROFILE_IMAGE_MUTATION, UpdateProfileImageData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "img": img });
    let data: UpdateProfileImageData =
        mutate(UPDATE_PROFILE_IMAGE_MUTATION, vars, Some(&token)).await?;

    if data.update_profile_image.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!(
            "Failed to update profile image: {}",
            data.update_profile_image.response_code
        )))
    }
}

/// Update the user's biography.
///
/// The biography text is encoded to base64 data URI format as expected by the backend.
///
/// # Arguments
///
/// * `biography` - Plain text biography content (max 5000 chars)
///
/// # Response Codes
///
/// - `11003`: Bio updated successfully
/// - `30101`: Missing fields
/// - `60501`: Not authenticated
#[server(UpdateBio, "/api")]
pub async fn update_bio(biography: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{UPDATE_BIO_MUTATION, UpdateBioData, mutate};
    use base64::Engine;

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    // Encode biography to base64 data URI format
    let encoded = base64::engine::general_purpose::STANDARD.encode(biography.as_bytes());
    let data_uri = format!("data:text/plain;base64,{}", encoded);

    let vars = serde_json::json!({ "biography": data_uri });
    let data: UpdateBioData = mutate(UPDATE_BIO_MUTATION, vars, Some(&token)).await?;

    if data.update_bio.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!(
            "Failed to update bio: {}",
            data.update_bio.response_code
        )))
    }
}

/// Update the user's username.
///
/// Requires password confirmation for security.
///
/// # Arguments
///
/// * `username` - New username (3-23 chars, alphanumeric/underscores/hyphens)
/// * `password` - Current password for confirmation
///
/// # Response Codes
///
/// - `11007`: Username changed successfully
/// - `30202`: Invalid username format
/// - `30101`: Missing fields
/// - `31001`: Password does not match
/// - `60501`: Not authenticated
#[server(UpdateUsername, "/api")]
pub async fn update_username(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{UPDATE_USERNAME_MUTATION, UpdateUsernameData, mutate};
    use crate::api::validation::validate_username;

    // Server-side validation
    validate_username(&username).map_err(|e| ServerFnError::new(e.to_string()))?;

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "username": username, "password": password });
    let data: UpdateUsernameData = mutate(UPDATE_USERNAME_MUTATION, vars, Some(&token)).await?;

    if data.update_username.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!(
            "Failed to update username: {}",
            data.update_username.response_code
        )))
    }
}

/// Update the user's password.
///
/// # Arguments
///
/// * `new_password` - The new password (min 8 chars, must meet strength requirements)
/// * `old_password` - Current password for verification
///
/// # Response Codes
///
/// - `11005`: Password updated successfully
/// - `30101`: Missing fields
/// - `31001`: Old password does not match
/// - `60501`: Not authenticated
#[server(UpdatePassword, "/api")]
pub async fn update_password(
    new_password: String,
    old_password: String,
) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{UPDATE_PASSWORD_MUTATION, UpdatePasswordData, mutate};
    use crate::api::validation::validate_password;

    // Server-side validation
    validate_password(&new_password).map_err(|e| ServerFnError::new(e.to_string()))?;

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "password": new_password, "expassword": old_password });
    let data: UpdatePasswordData = mutate(UPDATE_PASSWORD_MUTATION, vars, Some(&token)).await?;

    if data.update_password.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!(
            "Failed to update password: {}",
            data.update_password.response_code
        )))
    }
}

/// Update the user's email address.
///
/// Requires password confirmation for security.
///
/// # Arguments
///
/// * `email` - New email address
/// * `password` - Current password for confirmation
///
/// # Response Codes
///
/// - `11006`: Email updated successfully
/// - `30103`: Invalid email format
/// - `31001`: Password does not match
/// - `60501`: Not authenticated
#[server(UpdateEmail, "/api")]
pub async fn update_email(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{UPDATE_EMAIL_MUTATION, UpdateEmailData, mutate};
    use crate::api::validation::validate_email;

    // Server-side validation
    validate_email(&email).map_err(|e| ServerFnError::new(e.to_string()))?;

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "email": email, "password": password });
    let data: UpdateEmailData = mutate(UPDATE_EMAIL_MUTATION, vars, Some(&token)).await?;

    if data.update_email.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!(
            "Failed to update email: {}",
            data.update_email.response_code
        )))
    }
}

/// Update content filtering preferences.
///
/// # Arguments
///
/// * `severity_level` - Either "MYGRANDMALIKES" (strict) or "MYGRANDMAHATES" (lenient)
///
/// # Response Codes
///
/// - `11014`: Preferences updated successfully
/// - `60501`: Not authenticated
#[server(UpdateContentPreferences, "/api")]
pub async fn update_content_preferences(severity_level: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{UPDATE_PREFERENCES_MUTATION, UpdatePreferencesData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({
        "userPreferences": {
            "contentFilteringSeverityLevel": severity_level
        }
    });
    let data: UpdatePreferencesData =
        mutate(UPDATE_PREFERENCES_MUTATION, vars, Some(&token)).await?;

    if data.update_user_preferences.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!(
            "Failed to update preferences: {}",
            data.update_user_preferences.response_code
        )))
    }
}

/// Deactivate (soft-delete) the user's account.
///
/// This action sets the account status to 6 (deactivated) and cannot be undone.
/// Requires password confirmation for security.
///
/// # Arguments
///
/// * `password` - Current password for confirmation
///
/// # Response Codes
///
/// - `11012`: Account deleted successfully
/// - `30101`: Missing password
/// - `31001`: Password does not match
/// - `60501`: Not authenticated
#[server(DeleteAccount, "/api")]
pub async fn delete_account(password: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{DELETE_ACCOUNT_MUTATION, DeleteAccountData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = serde_json::json!({ "password": password });
    let data: DeleteAccountData = mutate(DELETE_ACCOUNT_MUTATION, vars, Some(&token)).await?;

    if data.delete_account.is_success() {
        Ok(())
    } else {
        Err(ServerFnError::new(format!(
            "Failed to delete account: {}",
            data.delete_account.response_code
        )))
    }
}
