use async_graphql::Object;

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    #[graphql(name = "_health")]
    async fn health(&self) -> bool {
        true
    }
}
