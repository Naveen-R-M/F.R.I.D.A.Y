use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents an access grant given by a user to a client application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessGrant {
    /// Unique identifier for the grant
    pub id: Uuid,
    
    /// User who granted access
    pub user_id: Uuid,
    
    /// Client/application that received access
    pub client_id: String,
    
    /// Scopes granted (e.g., "read", "write")
    pub scopes: Vec<String>,
    
    /// Context domains granted access to
    pub context_domains: Vec<String>,
    
    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new access grant
#[derive(Debug, Clone, Deserialize)]
pub struct GrantAccessInput {
    /// User who is granting access
    pub user_id: Uuid,
    
    /// Client/application receiving access
    pub client_id: String,
    
    /// Scopes to grant
    pub scopes: Vec<String>,
    
    /// Context domains to grant access to
    pub context_domains: Vec<String>,
    
    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

/// Audit log entry for consent-related actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentAuditLog {
    /// Unique identifier for the log entry
    pub id: Uuid,
    
    /// User who owns the data
    pub user_id: Uuid,
    
    /// Client/application that accessed data
    pub client_id: String,
    
    /// Type of action (grant, revoke, access)
    pub action: String,
    
    /// Additional details about the action
    pub details: serde_json::Value,
    
    /// Timestamp of the action
    pub timestamp: DateTime<Utc>,
}

/// Input for creating a new audit log entry
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAuditLogInput {
    /// User who owns the data
    pub user_id: Uuid,
    
    /// Client/application that accessed data
    pub client_id: String,
    
    /// Type of action (grant, revoke, access)
    pub action: String,
    
    /// Additional details about the action
    pub details: serde_json::Value,
}
