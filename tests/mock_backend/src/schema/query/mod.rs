pub mod ads;
pub mod chat;
pub mod comments;
pub mod health;
pub mod posts;
pub mod shop;
pub mod tokenomics;
pub mod users;
pub mod wallet;

use async_graphql::MergedObject;
use ads::AdQuery;
use chat::ChatQuery;
use comments::CommentQuery;
use health::HealthQuery;
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
);
