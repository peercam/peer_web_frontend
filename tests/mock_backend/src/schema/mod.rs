pub mod mutation;
pub mod query;

use async_graphql::{EmptySubscription, MergedObject, Schema};

use crate::state::SharedState;
use mutation::ads::AdMutation;
use mutation::auth::AuthMutation;
use mutation::chat::ChatMutation;
use mutation::comment::CommentMutation;
use mutation::post::PostMutation;
use mutation::profile::ProfileMutation;
use mutation::registration::RegistrationMutation;
use mutation::shop::ShopMutation;
use mutation::wallet::WalletMutation;
use query::QueryRoot;

/// Combined mutation root
#[derive(MergedObject, Default)]
pub struct MutationRoot(
    pub RegistrationMutation,
    pub AuthMutation,
    pub ProfileMutation,
    pub PostMutation,
    pub CommentMutation,
    pub ChatMutation,
    pub WalletMutation,
    pub AdMutation,
    pub ShopMutation,
);

/// The full GraphQL schema
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Build the GraphQL schema with injected state
pub fn build_schema(state: SharedState) -> AppSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(state)
    .finish()
}
