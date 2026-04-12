//! Page components (one per route).

pub mod dashboard;
pub mod login;
pub mod profile;
pub mod register;
pub mod view_post;
pub mod view_profile;

pub use dashboard::DashboardPage;
pub use login::LoginPage;
pub use profile::MyProfilePage;
pub use register::{RegisterPage, RegStep};
pub use view_post::ViewPostPage;
pub use view_profile::ViewProfilePage;
