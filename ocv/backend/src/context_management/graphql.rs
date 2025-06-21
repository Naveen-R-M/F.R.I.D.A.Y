use async_graphql::{Context, Object, ID, InputObject, Json};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

use super::models::{ContextShard, CreateShardInput, UpdateShardInput};
use super::service::ContextService;
use crate::api::AppState;

/// GraphQL representation of a context shard
#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct GraphQLContextShard {
    /// Unique identifier
    pub id: ID,
    /// Owner user ID
    pub user_id: ID,
    /// Domain or category
    pub domain: String,
    /// Type of content
    pub content_type: String,
    /// Vector representation for semantic search
    pub vector_representation: Option<Vec<f32>>,
    /// Metadata (non-encrypted)
    pub metadata: Json<HashMap<String, serde_json::Value>>,
    /// Creation timestamp
    pub created_at: async_graphql::DateTime,
    /// Last update timestamp
    pub updated_at: async_graphql::DateTime,
    /// Version number
    pub version: i32,
}

impl From<ContextShard> for GraphQLContextShard {
    fn from(shard: ContextShard) -> Self {
        Self {
            id: ID(shard.id.to_string()),
            user_id: ID(shard.user_id.to_string()),
            domain: shard.domain,
            content_type: shard.content_type,
            vector_representation: shard.vector_representation,
            metadata: shard.metadata,
            created_at: shard.created_at.into(),
            updated_at: shard.updated_at.into(),
            version: shard.version,
        }
    }
}

/// GraphQL input for creating a context shard
#[derive(InputObject)]
pub struct GraphQLCreateShardInput {
    /// User ID of the owner
    pub user_id: ID,
    /// Domain or category
    pub domain: String,
    /// Type of content
    pub content_type: String,
    /// Optional vector representation
    pub vector_representation: Option<Vec<f32>>,
    /// Metadata (non-encrypted)
    pub metadata: Json<HashMap<String, serde_json::Value>>,
    /// Content to be encrypted
    pub content: Json<serde_json::Value>,
}

impl From<GraphQLCreateShardInput> for CreateShardInput {
    fn from(input: GraphQLCreateShardInput) -> Self {
        Self {
            user_id: Uuid::parse_str(&input.user_id.0).unwrap(),
            domain: input.domain,
            content_type: input.content_type,
            vector_representation: input.vector_representation,
            metadata: input.metadata.0,
            content: input.content.0,
        }
    }
}

/// GraphQL input for updating a context shard
#[derive(InputObject)]
pub struct GraphQLUpdateShardInput {
    /// Optional new domain
    pub domain: Option<String>,
    /// Optional new content type
    pub content_type: Option<String>,
    /// Optional new vector representation
    pub vector_representation: Option<Vec<f32>>,
    /// Optional metadata updates (will be merged)
    pub metadata: Option<Json<HashMap<String, serde_json::Value>>>,
    /// Optional new content (will be encrypted)
    pub content: Option<Json<serde_json::Value>>,
    /// Current version for optimistic concurrency control
    pub current_version: i32,
}

impl From<GraphQLUpdateShardInput> for UpdateShardInput {
    fn from(input: GraphQLUpdateShardInput) -> Self {
        Self {
            domain: input.domain,
            content_type: input.content_type,
            vector_representation: input.vector_representation,
            metadata: input.metadata.map(|m| m.0),
            content: input.content.map(|c| c.0),
            current_version: input.current_version,
        }
    }
}

/// GraphQL response type for context shard with decrypted content
#[derive(async_graphql::SimpleObject)]
pub struct GraphQLShardWithContent {
    /// The shard metadata
    pub shard: GraphQLContextShard,
    /// Decrypted content
    pub content: Json<serde_json::Value>,
}

/// Context query root
#[derive(Default)]
pub struct ContextQuery;

#[Object]
impl ContextQuery {
    /// Get a context shard by ID
    async fn context_shard(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<GraphQLContextShard>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let uuid = Uuid::parse_str(&id.0)?;
        
        let shard = state.context_service.get_shard(uuid).await?;
        Ok(shard.map(GraphQLContextShard::from))
    }
    
    /// Get a context shard with decrypted content
    async fn context_shard_with_content(
        &self, 
        ctx: &Context<'_>, 
        id: ID
    ) -> async_graphql::Result<Option<GraphQLShardWithContent>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let uuid = Uuid::parse_str(&id.0)?;
        
        let result = state.context_service.get_shard_with_content(uuid).await?;
        
        Ok(result.map(|(shard, content)| GraphQLShardWithContent {
            shard: GraphQLContextShard::from(shard),
            content: Json(content),
        }))
    }
    
    /// Search for context shards
    async fn search_shards(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        query: String,
        domain: Option<String>,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<GraphQLContextShard>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let user_uuid = Uuid::parse_str(&user_id.0)?;
        
        let shards = state.context_service.search_shards(
            user_uuid, 
            &query, 
            domain.as_deref(), 
            limit.map(|l| l as i64)
        ).await?;
        
        Ok(shards.into_iter().map(GraphQLContextShard::from).collect())
    }
    
    /// Get all shards for a specific domain
    async fn shards_by_domain(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        domain: String,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<GraphQLContextShard>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let user_uuid = Uuid::parse_str(&user_id.0)?;
        
        let shards = state.context_service.get_shards_by_domain(
            user_uuid, 
            &domain, 
            limit.map(|l| l as i64)
        ).await?;
        
        Ok(shards.into_iter().map(GraphQLContextShard::from).collect())
    }
}

/// Context mutation root
#[derive(Default)]
pub struct ContextMutation;

#[Object]
impl ContextMutation {
    /// Create a new context shard
    async fn create_shard(
        &self,
        ctx: &Context<'_>,
        input: GraphQLCreateShardInput,
    ) -> async_graphql::Result<GraphQLContextShard> {
        let state = ctx.data::<Arc<AppState>>()?;
        
        let shard = state.context_service.create_shard(input.into()).await?;
        
        Ok(GraphQLContextShard::from(shard))
    }
    
    /// Update an existing context shard
    async fn update_shard(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: GraphQLUpdateShardInput,
    ) -> async_graphql::Result<Option<GraphQLContextShard>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let uuid = Uuid::parse_str(&id.0)?;
        
        let shard = state.context_service.update_shard(uuid, input.into()).await?;
        
        Ok(shard.map(GraphQLContextShard::from))
    }
    
    /// Delete a context shard
    async fn delete_shard(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>()?;
        let uuid = Uuid::parse_str(&id.0)?;
        
        let result = state.context_service.delete_shard(uuid).await?;
        
        Ok(result)
    }
}
