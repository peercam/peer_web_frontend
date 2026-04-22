//! Page components (one per route).

pub mod admin;
pub mod chat;
pub mod dashboard;
pub mod forgot_password;
pub mod invite;
pub mod login;
pub mod my_ads;
pub mod new_post;
pub mod peer_shop;
pub mod profile;
pub mod referral_board;
pub mod register;
pub mod settings;
pub mod version_history;
pub mod view_post;
pub mod view_profile;
pub mod wallet;

pub use admin::AdminPage;
pub use chat::ChatPage;
pub use dashboard::DashboardPage;
pub use forgot_password::ForgotPasswordPage;
pub use invite::InvitePage;
pub use login::LoginPage;
pub use my_ads::MyAdsPage;
pub use new_post::NewPostPage;
pub use peer_shop::PeerShopPage;
pub use profile::MyProfilePage;
pub use referral_board::{ReferralBoardPage, ReferralTab};
pub use register::{RegStep, RegisterPage};
pub use settings::{SettingsPage, SettingsTab};
pub use version_history::VersionHistoryPage;
pub use view_post::ViewPostPage;
pub use view_profile::ViewProfilePage;
pub use wallet::WalletPage;
