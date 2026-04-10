//! Page components (one per route).

pub mod login;
pub mod register;

pub use login::LoginPage;
pub use register::{RegisterPage, RegStep};
