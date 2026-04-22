//! New post components.
//!
//! This module contains all UI components for post creation:
//! - Content type tabs
//! - Form components for each content type
//! - Media upload and editing
//! - Tag management
//! - Preview

mod content_type_tabs;
mod form;
mod header;
mod media;
mod preview;
mod right_sidebar;
mod submit;
mod tags;

pub use content_type_tabs::ContentTypeTabs;
pub use form::NewPostForm;
pub use header::NewPostHeader;
pub use media::{ImageCropper, ImageSlider, VideoCover, VideoTrimmer, VoiceRecorder};
pub use preview::PostPreview;
pub use right_sidebar::NewPostRightSidebar;
pub use submit::SubmitButton;
pub use tags::{TagInput, TagList};
