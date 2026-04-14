pub mod health;
pub mod posts;
pub mod users;

use async_graphql::MergedObject;
use health::HealthQuery;
use posts::PostQuery;
use users::UserQuery;

#[derive(MergedObject, Default)]
pub struct QueryRoot(HealthQuery, UserQuery, PostQuery);
