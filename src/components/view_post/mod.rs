//! View post components for displaying a single post.
//!
//! This module provides components for the view post overlay/page:
//! - `PostMedia` - Image gallery, video, audio display
//! - `PostHeader` - Author info and follow button
//! - `PostContent` - Post title, description, tags
//! - `PostActions` - Like, dislike, save, share, report
//! - `Comments` - Comments list with infinite scroll
//! - `ShareModal` - Share dialog
//! - `ImageModal` - Fullscreen image viewer

mod comments;
mod image_modal;
mod post_actions;
mod post_content;
mod post_header;
mod post_media;
mod share_modal;

pub use comments::Comments;
pub use image_modal::ImageModal;
pub use post_actions::PostActions;
pub use post_content::PostContent;
pub use post_header::PostHeader;
pub use post_media::PostMedia;
pub use share_modal::ShareModal;
