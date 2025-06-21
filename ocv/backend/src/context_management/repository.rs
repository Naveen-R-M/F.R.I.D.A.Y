use anyhow::Result;
use uuid::Uuid;
use std::sync::Arc;

use super::models::{ContextShard, CreateShardInput, UpdateShardInput};
use crate::encryption::service::EncryptionService;
use crate::adapters::mem0::{Mem0Adapter, MemoryAdapter};

/// Repository for context shard storage and retrieval
pub struct ContextRepository {
    memory_adapter: Arc<dyn MemoryAdapter + Send + Sync>,
    encryption_service: Arc<EncryptionService>,
}

impl ContextRepository {
    /// Create a new context repository with mem0 adapter
    pub fn new_with_mem0(mem0_adapter: Arc<Mem0Adapter>, encryption_service: Arc<EncryptionService>) -> Self {
        Self { 
            memory_adapter: mem0_adapter,
            encryption_service,
        }
    }
    
    /// Create a new context shard
    pub async fn create_shard(&self, input: CreateShardInput) -> Result<ContextShard> {
        // Encrypt the content
        let content_json = serde_json::to_vec(&input.content)?;
        let encrypted_content = self.encryption_service.encrypt(
            &input.user_id.to_string(),
            &content_json
        ).await?;
        
        // Store in mem0
        self.memory_adapter.store_item(input, encrypted_content).await
    }
    
    /// Get a context shard by ID
    pub async fn get_shard_by_id(&self, id: Uuid) -> Result<Option<ContextShard>> {
        self.memory_adapter.get_item(&id.to_string()).await
    }
    
    /// Update an existing context shard
    pub async fn update_shard(&self, id: Uuid, input: UpdateShardInput) -> Result<Option<ContextShard>> {
        // Encrypt content if provided
        let encrypted_content = if let Some(content) = &input.content {
            let content_json = serde_json::to_vec(content)?;
            Some(self.encryption_service.encrypt(
                // Since we don't have user_id in the input, we need to get it from the current shard
                // In a real implementation, you would want to validate that the user_id matches
                &id.to_string(), // Using ID as key temporarily
                &content_json
            ).await?)
        } else {
            None
        };
        
        // Update in mem0
        self.memory_adapter.update_item(&id.to_string(), input, encrypted_content).await
    }
    
    /// Delete a context shard
    pub async fn delete_shard(&self, id: Uuid) -> Result<bool> {
        self.memory_adapter.delete_item(&id.to_string()).await
    }
    
    /// Search for context shards by text query
    pub async fn search_shards(
        &self, 
        user_id: Uuid,
        query: &str, 
        domain: Option<&str>,
        limit: Option<i64>
    ) -> Result<Vec<ContextShard>> {
        self.memory_adapter.search_items(
            &user_id.to_string(),
            query,
            domain,
            limit
        ).await
    }
    
    /// Get all shards for a specific domain
    pub async fn get_shards_by_domain(
        &self,
        user_id: Uuid,
        domain: &str,
        limit: Option<i64>
    ) -> Result<Vec<ContextShard>> {
        self.memory_adapter.get_items_by_domain(
            &user_id.to_string(),
            domain,
            limit
        ).await
    }
}
