//! Page components (one per route).

pub mod dashboard;
pub mod login;
pub mod register;

pub use dashboard::DashboardPage;
pub use login::LoginPage;
pub use register::{RegisterPage, RegStep};
