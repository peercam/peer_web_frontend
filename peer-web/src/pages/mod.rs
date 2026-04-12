//! Page components (one per route).

pub mod dashboard;
pub mod login;
pub mod register;
pub mod view_post;

pub use dashboard::DashboardPage;
pub use login::LoginPage;
pub use register::{RegisterPage, RegStep};
pub use view_post::ViewPostPage;
