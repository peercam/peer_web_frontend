//! Page components (one per route).

pub mod chat;
pub mod dashboard;
pub mod login;
pub mod new_post;
pub mod profile;
pub mod register;
pub mod view_post;
pub mod view_profile;
pub mod wallet;

pub use chat::ChatPage;
pub use dashboard::DashboardPage;
pub use login::LoginPage;
pub use new_post::NewPostPage;
pub use profile::MyProfilePage;
pub use register::{RegisterPage, RegStep};
pub use view_post::ViewPostPage;
pub use view_profile::ViewProfilePage;
pub use wallet::WalletPage;
