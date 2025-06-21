use async_graphql::{EmptySubscription, Schema, SchemaBuilder};

use crate::{
    context_management::graphql::{ContextMutation, ContextQuery},
    consent_manager::graphql::{ConsentMutation, ConsentQuery},
    identity::graphql::{IdentityMutation, IdentityQuery},
};

/// Root query object combining all query fields
#[derive(async_graphql::MergedObject, Default)]
pub struct Query(ContextQuery, ConsentQuery, IdentityQuery);

/// Root mutation object combining all mutation fields
#[derive(async_graphql::MergedObject, Default)]
pub struct Mutation(ContextMutation, ConsentMutation, IdentityMutation);

/// Create the GraphQL schema with all queries and mutations
pub type OcvSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema() -> OcvSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .enable_federation()
        .finish()
}

/// Build a schema with custom configuration
pub fn schema_builder() -> SchemaBuilder<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .enable_federation()
}
