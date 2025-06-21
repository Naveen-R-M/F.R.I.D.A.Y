use async_graphql::{Context, Object, ID, InputObject};
use uuid::Uuid;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use super::models::{AccessGrant, GrantAccessInput, ConsentAuditLog};
use super::service::ConsentManager;
use crate::api::AppState;

/// GraphQL representation of an access grant
#[derive(async_graphql::SimpleObject)]
pub struct GraphQLAccessGrant {
    /// Unique identifier
    pub id: ID,
    /// Owner user ID
    pub user_id: ID,
    /// Client/application ID
    pub client_id: String,
    /// Granted scopes
    pub scopes: Vec<String>,
    /// Granted context domains
    pub context_domains: Vec<String>,
    /// Expiration time (if any)
    pub expires_at: Option<async_graphql::DateTime>,
    /// Creation timestamp
    pub created_at: async_graphql::DateTime,
}

impl From<AccessGrant> for GraphQLAccessGrant {
    fn from(grant: AccessGrant) -> Self {
        Self {
            id: ID(grant.id.to_string()),
            user_id: ID(grant.user_id.to_string()),
            client_id: grant.client_id,
            scopes: grant.scopes,
            context_domains: grant.context_domains,
            expires_at: grant.expires_at.map(Into::into),
            created_at: grant.created_at.into(),
        }
    }
}

/// GraphQL input for granting access
#[derive(InputObject)]
pub struct GraphQLGrantAccessInput {
    /// User granting access
    pub user_id: ID,
    /// Client receiving access
    pub client_id: String,
    /// Scopes to grant
    pub scopes: Vec<String>,
    /// Context domains to grant access to
    pub context_domains: Vec<String>,
    /// Optional expiration time
    pub expires_at: Option<async_graphql::DateTime>,
}

impl From<GraphQLGrantAccessInput> for GrantAccessInput {
    fn from(input: GraphQLGrantAccessInput) -> Self {
        Self {
            user_id: Uuid::parse_str(&input.user_id.0).unwrap(),
            client_id: input.client_id,
            scopes: input.scopes,
            context_domains: input.context_domains,
            expires_at: input.expires_at.map(|dt| dt.into()),
        }
    }
}

/// GraphQL representation of a consent audit log entry
#[derive(async_graphql::SimpleObject)]
pub struct GraphQLConsentAuditLog {
    /// Unique identifier
    pub id: ID,
    /// User who owns the data
    pub user_id: ID,
    /// Client/application that accessed data
    pub client_id: String,
    /// Type of action
    pub action: String,
    /// Additional details
    pub details: async_graphql::Json<serde_json::Value>,
    /// Timestamp
    pub timestamp: async_graphql::DateTime,
}

impl From<ConsentAuditLog> for GraphQLConsentAuditLog {
    fn from(log: ConsentAuditLog) -> Self {
        Self {
            id: ID(log.id.to_string()),
            user_id: ID(log.user_id.to_string()),
            client_id: log.client_id,
            action: log.action,
            details: async_graphql::Json(log.details),
            timestamp: log.timestamp.into(),
        }
    }
}

/// Consent query root
#[derive(Default)]
pub struct ConsentQuery;

#[Object]
impl ConsentQuery {
    /// Get all active grants for a user
    async fn active_grants(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
    ) -> async_graphql::Result<Vec<GraphQLAccessGrant>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let user_uuid = Uuid::parse_str(&user_id.0)?;
        
        let grants = state.consent_manager.get_active_grants(user_uuid).await?;
        
        Ok(grants.into_iter().map(GraphQLAccessGrant::from).collect())
    }
    
    /// Check if a client has access to a specific domain
    async fn check_access(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        client_id: String,
        domain: String,
        scope: String,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>()?;
        let user_uuid = Uuid::parse_str(&user_id.0)?;
        
        let has_access = state.consent_manager.check_access(
            user_uuid,
            &client_id,
            &domain,
            &scope,
        ).await?;
        
        Ok(has_access)
    }
    
    /// Get audit logs for a user
    async fn audit_logs(
        &self,
        ctx: &Context<'_>,
        user_id: ID,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> async_graphql::Result<Vec<GraphQLConsentAuditLog>> {
        let state = ctx.data::<Arc<AppState>>()?;
        let user_uuid = Uuid::parse_str(&user_id.0)?;
        
        let logs = state.consent_manager.get_audit_logs(
            user_uuid,
            limit.map(|l| l as i64),
            offset.map(|o| o as i64),
        ).await?;
        
        Ok(logs.into_iter().map(GraphQLConsentAuditLog::from).collect())
    }
}

/// Consent mutation root
#[derive(Default)]
pub struct ConsentMutation;

#[Object]
impl ConsentMutation {
    /// Grant access to a client
    async fn grant_access(
        &self,
        ctx: &Context<'_>,
        input: GraphQLGrantAccessInput,
    ) -> async_graphql::Result<GraphQLAccessGrant> {
        let state = ctx.data::<Arc<AppState>>()?;
        
        let grant = state.consent_manager.grant_access(input.into()).await?;
        
        Ok(GraphQLAccessGrant::from(grant))
    }
    
    /// Revoke an access grant
    async fn revoke_access(
        &self,
        ctx: &Context<'_>,
        grant_id: ID,
        user_id: ID,
        client_id: String,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>()?;
        let grant_uuid = Uuid::parse_str(&grant_id.0)?;
        let user_uuid = Uuid::parse_str(&user_id.0)?;
        
        let result = state.consent_manager.revoke_grant(
            grant_uuid,
            user_uuid,
            &client_id,
        ).await?;
        
        Ok(result)
    }
}
