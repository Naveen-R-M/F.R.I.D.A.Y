use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use super::{
    models::{AccessGrant, GrantAccessInput, ConsentAuditLog, CreateAuditLogInput},
    repository::ConsentRepository,
};
use crate::policy_engine::service::PolicyEngine;

/// Service for managing consent and access grants
pub struct ConsentManager {
    repository: ConsentRepository,
    policy_engine: Arc<PolicyEngine>,
}

impl ConsentManager {
    /// Create a new consent manager
    pub fn new(pool: PgPool, policy_engine: Arc<PolicyEngine>) -> Arc<Self> {
        Arc::new(Self {
            repository: ConsentRepository::new(pool),
            policy_engine,
        })
    }
    
    /// Grant access to a client
    pub async fn grant_access(&self, input: GrantAccessInput) -> Result<AccessGrant> {
        // First, check if the policy allows this grant
        let policy_input = serde_json::json!({
            "user": input.user_id.to_string(),
            "action": "grant",
            "client": input.client_id,
            "domains": input.context_domains,
            "scopes": input.scopes
        });
        
        let allowed = self.policy_engine.evaluate("consent", policy_input).await?;
        
        if !allowed {
            return Err(anyhow::anyhow!("Policy denied this access grant"));
        }
        
        // Create the grant
        let grant = self.repository.create_grant(input.clone()).await?;
        
        // Log the action
        let audit_input = CreateAuditLogInput {
            user_id: input.user_id,
            client_id: input.client_id,
            action: "grant".to_string(),
            details: serde_json::json!({
                "grant_id": grant.id.to_string(),
                "scopes": input.scopes,
                "domains": input.context_domains,
                "expires_at": input.expires_at
            }),
        };
        
        self.repository.create_audit_log(audit_input).await?;
        
        Ok(grant)
    }
    
    /// Get all active grants for a user
    pub async fn get_active_grants(&self, user_id: Uuid) -> Result<Vec<AccessGrant>> {
        self.repository.get_active_grants(user_id).await
    }
    
    /// Revoke an access grant
    pub async fn revoke_grant(&self, grant_id: Uuid, user_id: Uuid, client_id: &str) -> Result<bool> {
        // First check if the grant exists and belongs to the user
        let grants = self.repository.get_active_grants(user_id).await?;
        let grant = grants.iter().find(|g| g.id == grant_id);
        
        let Some(grant) = grant else {
            return Ok(false);
        };
        
        // Check policy
        let policy_input = serde_json::json!({
            "user": user_id.to_string(),
            "action": "revoke",
            "client": client_id,
            "grant_id": grant_id.to_string()
        });
        
        let allowed = self.policy_engine.evaluate("consent", policy_input).await?;
        
        if !allowed {
            return Err(anyhow::anyhow!("Policy denied this revocation"));
        }
        
        // Revoke the grant
        let result = self.repository.revoke_grant(grant_id).await?;
        
        if result {
            // Log the action
            let audit_input = CreateAuditLogInput {
                user_id,
                client_id: client_id.to_string(),
                action: "revoke".to_string(),
                details: serde_json::json!({
                    "grant_id": grant_id.to_string(),
                    "scopes": grant.scopes,
                    "domains": grant.context_domains
                }),
            };
            
            self.repository.create_audit_log(audit_input).await?;
        }
        
        Ok(result)
    }
    
    /// Check if a client has access to a specific domain for a user
    pub async fn check_access(
        &self,
        user_id: Uuid,
        client_id: &str,
        domain: &str,
        required_scope: &str,
    ) -> Result<bool> {
        let has_access = self.repository.check_access(user_id, client_id, domain, required_scope).await?;
        
        if has_access {
            // Log the access
            let audit_input = CreateAuditLogInput {
                user_id,
                client_id: client_id.to_string(),
                action: "access".to_string(),
                details: serde_json::json!({
                    "domain": domain,
                    "scope": required_scope,
                    "success": true
                }),
            };
            
            self.repository.create_audit_log(audit_input).await?;
        }
        
        Ok(has_access)
    }
    
    /// Get audit logs for a user
    pub async fn get_audit_logs(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ConsentAuditLog>> {
        self.repository.get_audit_logs(user_id, limit, offset).await
    }
}
