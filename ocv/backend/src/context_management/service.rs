use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;
use serde_json::Value;

use super::{
    models::{ContextShard, CreateShardInput, UpdateShardInput},
    repository::ContextRepository,
};
use crate::encryption::service::EncryptionService;
use crate::adapters::mem0::{Mem0Adapter, Mem0Config};

/// Service for managing context shards
pub struct ContextService {
    repository: ContextRepository,
    encryption_service: Arc<EncryptionService>,
}

impl ContextService {
    /// Create a new context service with mem0
    pub fn new_with_mem0(encryption_service: Arc<EncryptionService>) -> Arc<Self> {
        // Create mem0 adapter
        let mem0_config = Mem0Config::default();
        let mem0_adapter = Mem0Adapter::new(mem0_config);
        
        Arc::new(Self {
            repository: ContextRepository::new_with_mem0(mem0_adapter, encryption_service.clone()),
            encryption_service,
        })
    }
    
    /// Create a new context shard
    pub async fn create_shard(&self, input: CreateShardInput) -> Result<ContextShard> {
        self.repository.create_shard(input).await
    }
    
    /// Get a context shard by ID
    pub async fn get_shard(&self, id: Uuid) -> Result<Option<ContextShard>> {
        self.repository.get_shard_by_id(id).await
    }
    
    /// Get a context shard with decrypted content
    pub async fn get_shard_with_content(&self, id: Uuid) -> Result<Option<(ContextShard, Value)>> {
        let shard = match self.repository.get_shard_by_id(id).await? {
            Some(s) => s,
            None => return Ok(None),
        };
        
        // Decrypt the content
        let decrypted = self.encryption_service.decrypt(
            &shard.user_id.to_string(),
            &shard.content
        ).await?;
        
        let content: Value = serde_json::from_slice(&decrypted)?;
        
        Ok(Some((shard, content)))
    }
    
    /// Update an existing context shard
    pub async fn update_shard(&self, id: Uuid, input: UpdateShardInput) -> Result<Option<ContextShard>> {
        self.repository.update_shard(id, input).await
    }
    
    /// Delete a context shard
    pub async fn delete_shard(&self, id: Uuid) -> Result<bool> {
        self.repository.delete_shard(id).await
    }
    
    /// Search for context shards
    pub async fn search_shards(
        &self,
        user_id: Uuid,
        query: &str,
        domain: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<ContextShard>> {
        self.repository.search_shards(user_id, query, domain, limit).await
    }
    
    /// Get all shards for a specific domain
    pub async fn get_shards_by_domain(
        &self,
        user_id: Uuid,
        domain: &str,
        limit: Option<i64>,
    ) -> Result<Vec<ContextShard>> {
        self.repository.get_shards_by_domain(user_id, domain, limit).await
    }
}
