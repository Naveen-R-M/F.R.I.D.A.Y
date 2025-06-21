use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::models::{AccessGrant, GrantAccessInput, ConsentAuditLog, CreateAuditLogInput};

/// Repository for consent-related data storage and retrieval
pub struct ConsentRepository {
    pool: PgPool,
}

impl ConsentRepository {
    /// Create a new consent repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Create a new access grant
    pub async fn create_grant(&self, input: GrantAccessInput) -> Result<AccessGrant> {
        let grant = sqlx::query_as!(
            AccessGrant,
            r#"
            INSERT INTO access_grants (
                user_id, client_id, scopes, context_domains, expires_at
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                id, user_id, client_id, scopes, context_domains,
                expires_at, created_at
            "#,
            input.user_id,
            input.client_id,
            &input.scopes as &[String],
            &input.context_domains as &[String],
            input.expires_at,
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(grant)
    }
    
    /// Get all active grants for a user
    pub async fn get_active_grants(&self, user_id: Uuid) -> Result<Vec<AccessGrant>> {
        let grants = sqlx::query_as!(
            AccessGrant,
            r#"
            SELECT 
                id, user_id, client_id, scopes, context_domains,
                expires_at, created_at
            FROM access_grants
            WHERE user_id = $1
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(grants)
    }
    
    /// Revoke an access grant
    pub async fn revoke_grant(&self, grant_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM access_grants
            WHERE id = $1
            "#,
            grant_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    /// Check if a client has access to a specific domain for a user
    pub async fn check_access(
        &self,
        user_id: Uuid,
        client_id: &str,
        domain: &str,
        required_scope: &str,
    ) -> Result<bool> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*)
            FROM access_grants
            WHERE user_id = $1
              AND client_id = $2
              AND $3 = ANY(context_domains)
              AND $4 = ANY(scopes)
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
            user_id,
            client_id,
            domain,
            required_scope
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(count.count.unwrap_or(0) > 0)
    }
    
    /// Create an audit log entry
    pub async fn create_audit_log(&self, input: CreateAuditLogInput) -> Result<ConsentAuditLog> {
        let log = sqlx::query_as!(
            ConsentAuditLog,
            r#"
            INSERT INTO consent_audit_logs (
                user_id, client_id, action, details
            )
            VALUES ($1, $2, $3, $4)
            RETURNING 
                id, user_id, client_id, action, 
                details as "details: serde_json::Value",
                timestamp
            "#,
            input.user_id,
            input.client_id,
            input.action,
            input.details as serde_json::Value,
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(log)
    }
    
    /// Get audit logs for a user
    pub async fn get_audit_logs(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ConsentAuditLog>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let logs = sqlx::query_as!(
            ConsentAuditLog,
            r#"
            SELECT 
                id, user_id, client_id, action, 
                details as "details: serde_json::Value",
                timestamp
            FROM consent_audit_logs
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(logs)
    }
}
