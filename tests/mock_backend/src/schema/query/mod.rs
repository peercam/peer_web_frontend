pub mod admin;
pub mod admin_gems;
pub mod ads;
pub mod chat;
pub mod comments;
pub mod health;
pub mod moderation;
pub mod posts;
pub mod shop;
pub mod tokenomics;
pub mod users;
pub mod wallet;

use admin::AdminQuery;
use admin_gems::AdminGemQuery;
use ads::AdQuery;
use async_graphql::MergedObject;
use chat::ChatQuery;
use comments::CommentQuery;
use health::HealthQuery;
use moderation::ModerationQuery;
use posts::PostQuery;
use shop::ShopQuery;
use tokenomics::TokenomicsQuery;
use users::UserQuery;
use wallet::WalletQuery;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    HealthQuery,
    UserQuery,
    PostQuery,
    CommentQuery,
    ChatQuery,
    WalletQuery,
    TokenomicsQuery,
    AdQuery,
    ShopQuery,
    ModerationQuery,
    AdminQuery,
    AdminGemQuery,
);
