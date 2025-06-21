use async_graphql::{Context, Object, ID, InputObject};
use uuid::Uuid;
use std::sync::Arc;

use super::models::{User, CreateUserInput, Credentials, AuthToken};
use super::service::IdentityService;
use crate::api::AppState;

/// GraphQL representation of a user
#[derive(async_graphql::SimpleObject)]
pub struct GraphQLUser {
    /// Unique identifier
    pub id: ID,
    /// Email address
    pub email: String,
    /// Display name
    pub display_name: String,
    /// Creation timestamp
    pub created_at: async_graphql::DateTime,
    /// Last update timestamp
    pub updated_at: async_graphql::DateTime,
}

impl From<User> for GraphQLUser {
    fn from(user: User) -> Self {
        Self {
            id: ID(user.id.to_string()),
            email: user.email,
            display_name: user.display_name,
            created_at: user.created_at.into(),
            updated_at: user.updated_at.into(),
        }
    }
}

/// GraphQL representation of an auth token
#[derive(async_graphql::SimpleObject)]
pub struct GraphQLAuthToken {
    /// JWT token
    pub token: String,
    /// Expiration time
    pub expires_at: async_graphql::DateTime,
    /// User information
    pub user: GraphQLUser,
}

impl From<AuthToken> for GraphQLAuthToken {
    fn from(token: AuthToken) -> Self {
        Self {
            token: token.token,
            expires_at: token.expires_at.into(),
            user: GraphQLUser::from(token.user),
        }
    }
}

/// GraphQL input for creating a user
#[derive(InputObject)]
pub struct GraphQLCreateUserInput {
    /// Email address
    pub email: String,
    /// Display name
    pub display_name: String,
    /// Password
    pub password: String,
}

impl From<GraphQLCreateUserInput> for CreateUserInput {
    fn from(input: GraphQLCreateUserInput) -> Self {
        Self {
            email: input.email,
            display_name: input.display_name,
            password: input.password,
        }
    }
}

/// GraphQL input for authentication
#[derive(InputObject)]
pub struct GraphQLCredentials {
    /// Email address
    pub email: String,
    /// Password
    pub password: String,
}

impl From<GraphQLCredentials> for Credentials {
    fn from(input: GraphQLCredentials) -> Self {
        Self {
            email: input.email,
            password: input.password,
        }
    }
}

/// Identity query root
#[derive(Default)]
pub struct IdentityQuery;

#[Object]
impl IdentityQuery {
    /// Get the current user from JWT token
    async fn me(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> async_graphql::Result<Option<GraphQLUser>> {
        let state = ctx.data::<Arc<AppState>>()?;
        
        let user_id = match state.identity_service.validate_token(&token).await? {
            Some(id) => id,
            None => return Ok(None),
        };
        
        let user = state.identity_service.get_user(user_id).await?;
        
        Ok(user.map(GraphQLUser::from))
    }
    
    /// Get a user by ID
    async fn user(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<GraphQLUser>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let user_uuid = Uuid::parse_str(&id.0)?;
        
        let user = state.identity_service.get_user(user_uuid).await?;
        
        Ok(user.map(GraphQLUser::from))
    }
}

/// Identity mutation root
#[derive(Default)]
pub struct IdentityMutation;

#[Object]
impl IdentityMutation {
    /// Register a new user
    async fn register(
        &self,
        ctx: &Context<'_>,
        input: GraphQLCreateUserInput,
    ) -> async_graphql::Result<GraphQLUser> {
        let state = ctx.data::<Arc<AppState>>()?;
        
        let user = state.identity_service.create_user(input.into()).await?;
        
        Ok(GraphQLUser::from(user))
    }
    
    /// Login with email and password
    async fn login(
        &self,
        ctx: &Context<'_>,
        credentials: GraphQLCredentials,
    ) -> async_graphql::Result<Option<GraphQLAuthToken>> {
        let state = ctx.data::<Arc<AppState>>()?;
        
        let token = state.identity_service.authenticate(credentials.into()).await?;
        
        Ok(token.map(GraphQLAuthToken::from))
    }
    
    /// Update user profile
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        id: ID,
        display_name: Option<String>,
    ) -> async_graphql::Result<Option<GraphQLUser>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let user_uuid = Uuid::parse_str(&id.0)?;
        
        let user = state.identity_service.update_user(user_uuid, display_name).await?;
        
        Ok(user.map(GraphQLUser::from))
    }
}
