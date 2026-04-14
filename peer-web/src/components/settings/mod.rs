//! Settings page components.
//!
//! This module contains all the UI components for the settings page:
//! - Tab navigation menu
//! - Profile settings (avatar, bio, username, password, email)
//! - Content filtering settings
//! - Notification and preferences stubs

pub mod change_email;
pub mod change_password;
pub mod change_username;
pub mod content;
pub mod image_modal;
pub mod menu;
pub mod notification;
pub mod preferences;
pub mod profile;

pub use change_email::ChangeEmailPanel;
pub use change_password::ChangePasswordPanel;
pub use change_username::ChangeUsernamePanel;
pub use content::ContentSettings;
pub use image_modal::ImageUploadModal;
pub use menu::SettingsMenu;
pub use notification::NotificationSettings;
pub use preferences::PreferencesSettings;
pub use profile::ProfileSettings;
