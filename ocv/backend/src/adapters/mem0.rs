use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use reqwest::Client;
use async_trait::async_trait;

use crate::context_management::models::{ContextShard, CreateShardInput, UpdateShardInput};

/// Configuration for mem0 client
#[derive(Clone, Debug)]
pub struct Mem0Config {
    /// Base URL for the mem0 API
    pub base_url: String,
    /// API key for authentication
    pub api_key: String,
}

impl Default for Mem0Config {
    fn default() -> Self {
        Self {
            base_url: "https://api.basic.tech/mem0".to_string(),
            api_key: std::env::var("BASIC_MEM0_API_KEY").unwrap_or_default(),
        }
    }
}

/// mem0 memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mem0Item {
    /// Unique identifier
    pub id: String,
    /// User identifier
    pub user_id: String,
    /// Content domain/category
    pub domain: String,
    /// Type of content
    pub content_type: String,
    /// Vector embedding for semantic search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    /// Metadata for filtering and organization
    pub metadata: serde_json::Value,
    /// Encrypted content
    pub content: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
    /// Version number for concurrency control
    pub version: i32,
}

/// Adapter for integrating with Basic's mem0
pub struct Mem0Adapter {
    config: Mem0Config,
    client: Client,
}

#[async_trait]
pub trait MemoryAdapter {
    /// Store a memory item
    async fn store_item(&self, input: CreateShardInput, encrypted_content: Vec<u8>) -> Result<ContextShard>;
    
    /// Retrieve a memory item
    async fn get_item(&self, id: &str) -> Result<Option<ContextShard>>;
    
    /// Update a memory item
    async fn update_item(&self, id: &str, input: UpdateShardInput, encrypted_content: Option<Vec<u8>>) -> Result<Option<ContextShard>>;
    
    /// Delete a memory item
    async fn delete_item(&self, id: &str) -> Result<bool>;
    
    /// Search for memory items
    async fn search_items(
        &self, 
        user_id: &str,
        query: &str, 
        domain: Option<&str>,
        limit: Option<i64>
    ) -> Result<Vec<ContextShard>>;
    
    /// Get items by domain
    async fn get_items_by_domain(
        &self,
        user_id: &str,
        domain: &str,
        limit: Option<i64>
    ) -> Result<Vec<ContextShard>>;
}

impl Mem0Adapter {
    /// Create a new mem0 adapter
    pub fn new(config: Mem0Config) -> Arc<Self> {
        Arc::new(Self {
            config,
            client: Client::new(),
        })
    }
    
    /// Convert a mem0 item to a context shard
    fn to_context_shard(&self, item: Mem0Item) -> Result<ContextShard> {
        Ok(ContextShard {
            id: Uuid::parse_str(&item.id)?,
            user_id: Uuid::parse_str(&item.user_id)?,
            domain: item.domain,
            content_type: item.content_type,
            vector_representation: item.embedding,
            metadata: sqlx::types::Json(
                item.metadata
                    .as_object()
                    .unwrap_or(&serde_json::Map::new())
                    .clone()
            ),
            content: base64::decode(&item.content)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&item.created_at)?.into(),
            updated_at: chrono::DateTime::parse_from_rfc3339(&item.updated_at)?.into(),
            version: item.version,
        })
    }
    
    /// Convert a context shard to a mem0 item
    fn to_mem0_item(&self, shard: &ContextShard) -> Result<Mem0Item> {
        Ok(Mem0Item {
            id: shard.id.to_string(),
            user_id: shard.user_id.to_string(),
            domain: shard.domain.clone(),
            content_type: shard.content_type.clone(),
            embedding: shard.vector_representation.clone(),
            metadata: serde_json::Value::Object(shard.metadata.0.clone()),
            content: base64::encode(&shard.content),
            created_at: shard.created_at.to_rfc3339(),
            updated_at: shard.updated_at.to_rfc3339(),
            version: shard.version,
        })
    }
}

#[async_trait]
impl MemoryAdapter for Mem0Adapter {
    async fn store_item(&self, input: CreateShardInput, encrypted_content: Vec<u8>) -> Result<ContextShard> {
        let url = format!("{}/memory", self.config.base_url);
        
        let item = Mem0Item {
            id: Uuid::new_v4().to_string(),
            user_id: input.user_id.to_string(),
            domain: input.domain.clone(),
            content_type: input.content_type.clone(),
            embedding: input.vector_representation.clone(),
            metadata: serde_json::Value::Object(input.metadata.clone()),
            content: base64::encode(&encrypted_content),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            version: 1,
        };
        
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&item)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to store item: {}", response.status()));
        }
        
        let created_item: Mem0Item = response.json().await?;
        self.to_context_shard(created_item)
    }
    
    async fn get_item(&self, id: &str) -> Result<Option<ContextShard>> {
        let url = format!("{}/memory/{}", self.config.base_url, id);
        
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;
        
        if response.status().is_client_error() {
            return Ok(None);
        }
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get item: {}", response.status()));
        }
        
        let item: Mem0Item = response.json().await?;
        Ok(Some(self.to_context_shard(item)?))
    }
    
    async fn update_item(&self, id: &str, input: UpdateShardInput, encrypted_content: Option<Vec<u8>>) -> Result<Option<ContextShard>> {
        // First get the current item
        let current = match self.get_item(id).await? {
            Some(item) => item,
            None => return Ok(None),
        };
        
        // Check version
        if current.version != input.current_version {
            return Err(anyhow::anyhow!("Version mismatch"));
        }
        
        let url = format!("{}/memory/{}", self.config.base_url, id);
        
        // Prepare update payload
        let mut update_data = serde_json::Map::new();
        
        if let Some(domain) = input.domain {
            update_data.insert("domain".to_string(), serde_json::Value::String(domain));
        }
        
        if let Some(content_type) = input.content_type {
            update_data.insert("content_type".to_string(), serde_json::Value::String(content_type));
        }
        
        if let Some(embedding) = input.vector_representation {
            update_data.insert("embedding".to_string(), serde_json::to_value(embedding)?);
        }
        
        if let Some(metadata) = input.metadata {
            update_data.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }
        
        if let Some(content) = encrypted_content {
            update_data.insert("content".to_string(), serde_json::Value::String(base64::encode(&content)));
        }
        
        // Always update version and timestamp
        update_data.insert("version".to_string(), serde_json::Value::Number((current.version + 1).into()));
        update_data.insert("updated_at".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        
        let response = self.client.patch(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&update_data)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to update item: {}", response.status()));
        }
        
        let updated_item: Mem0Item = response.json().await?;
        Ok(Some(self.to_context_shard(updated_item)?))
    }
    
    async fn delete_item(&self, id: &str) -> Result<bool> {
        let url = format!("{}/memory/{}", self.config.base_url, id);
        
        let response = self.client.delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;
        
        if response.status().is_client_error() {
            return Ok(false);
        }
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to delete item: {}", response.status()));
        }
        
        Ok(true)
    }
    
    async fn search_items(
        &self, 
        user_id: &str,
        query: &str, 
        domain: Option<&str>,
        limit: Option<i64>
    ) -> Result<Vec<ContextShard>> {
        let mut url = format!("{}/memory/search?user_id={}&q={}", 
            self.config.base_url, 
            user_id,
            urlencoding::encode(query)
        );
        
        if let Some(d) = domain {
            url.push_str(&format!("&domain={}", urlencoding::encode(d)));
        }
        
        if let Some(l) = limit {
            url.push_str(&format!("&limit={}", l));
        }
        
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to search items: {}", response.status()));
        }
        
        let items: Vec<Mem0Item> = response.json().await?;
        let mut shards = Vec::with_capacity(items.len());
        
        for item in items {
            shards.push(self.to_context_shard(item)?);
        }
        
        Ok(shards)
    }
    
    async fn get_items_by_domain(
        &self,
        user_id: &str,
        domain: &str,
        limit: Option<i64>
    ) -> Result<Vec<ContextShard>> {
        let mut url = format!("{}/memory?user_id={}&domain={}", 
            self.config.base_url, 
            user_id,
            urlencoding::encode(domain)
        );
        
        if let Some(l) = limit {
            url.push_str(&format!("&limit={}", l));
        }
        
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get items by domain: {}", response.status()));
        }
        
        let items: Vec<Mem0Item> = response.json().await?;
        let mut shards = Vec::with_capacity(items.len());
        
        for item in items {
            shards.push(self.to_context_shard(item)?);
        }
        
        Ok(shards)
    }
}
