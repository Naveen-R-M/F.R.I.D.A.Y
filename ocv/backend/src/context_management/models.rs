use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Represents a single context shard in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextShard {
    /// Unique identifier for the shard
    pub id: Uuid,
    
    /// User who owns this shard
    pub user_id: Uuid,
    
    /// Domain or category (e.g., "travel-preferences", "shopping-history")
    pub domain: String,
    
    /// Type of content (e.g., "preferences", "history", "profile")
    pub content_type: String,
    
    /// Vector representation for semantic search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_representation: Option<Vec<f32>>,
    
    /// Metadata for the shard (non-encrypted)
    pub metadata: Json<HashMap<String, serde_json::Value>>,
    
    /// Encrypted content of the shard
    pub content: Vec<u8>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Version number for optimistic concurrency control
    pub version: i32,
}

/// Input for creating a new context shard
#[derive(Debug, Clone, Deserialize)]
pub struct CreateShardInput {
    /// User who owns this shard
    pub user_id: Uuid,
    
    /// Domain or category
    pub domain: String,
    
    /// Type of content
    pub content_type: String,
    
    /// Optional vector representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_representation: Option<Vec<f32>>,
    
    /// Metadata (non-encrypted)
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Content to be encrypted
    pub content: serde_json::Value,
}

/// Input for updating an existing context shard
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateShardInput {
    /// Optional new domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    
    /// Optional new content type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    
    /// Optional new vector representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_representation: Option<Vec<f32>>,
    
    /// Optional metadata updates (will be merged)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    
    /// Optional new content (will be encrypted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    
    /// Current version for optimistic concurrency control
    pub current_version: i32,
}
