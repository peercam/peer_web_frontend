//! Shared layout shell — header, mobile footer, and sidebar rails.
//!
//! See [`docs/plans/layout/layout-shell-implementation.md`] for the design
//! document. The components in this module replace ten+ private
//! `fn MobileFooter` copies and the inline `<header>` / `<aside>` blocks
//! that every authenticated page used to carry.

pub mod left_rail;
pub mod mobile_footer;
pub mod right_rail;
pub mod site_header;
pub mod site_shell;

pub use left_rail::LeftRail;
pub use mobile_footer::MobileFooter;
pub use right_rail::{RightRail, StandardRightRail};
pub use site_header::{HeaderSpelling, SiteHeader};
pub use site_shell::SiteShell;
