use anyhow::Result;
use sodiumoxide::crypto::secretbox;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Service for encrypting and decrypting data
#[derive(Clone)]
pub struct EncryptionService {
    // In a real implementation, keys would be stored securely and possibly fetched from a KMS
    // This is a simplified version for demonstration purposes
    user_keys: Arc<RwLock<HashMap<String, secretbox::Key>>>,
}

impl EncryptionService {
    /// Create a new encryption service
    pub fn new() -> Arc<Self> {
        // Initialize sodiumoxide
        sodiumoxide::init().expect("Failed to initialize sodiumoxide");
        
        Arc::new(Self {
            user_keys: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get or create a key for a user
    fn get_or_create_key(&self, user_id: &str) -> Result<secretbox::Key> {
        // Check if we already have a key for this user
        {
            let keys = self.user_keys.read().unwrap();
            if let Some(key) = keys.get(user_id) {
                return Ok(key.clone());
            }
        }
        
        // Create a new key
        let key = secretbox::gen_key();
        
        // Store the key
        {
            let mut keys = self.user_keys.write().unwrap();
            keys.insert(user_id.to_string(), key.clone());
        }
        
        Ok(key)
    }
    
    /// Encrypt data for a user
    pub async fn encrypt(&self, user_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let key = self.get_or_create_key(user_id)?;
        let nonce = secretbox::gen_nonce();
        
        // Encrypt the data
        let encrypted = secretbox::seal(data, &nonce, &key);
        
        // Combine nonce and encrypted data
        let mut result = nonce.as_ref().to_vec();
        result.extend_from_slice(&encrypted);
        
        Ok(result)
    }
    
    /// Decrypt data for a user
    pub async fn decrypt(&self, user_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < secretbox::NONCEBYTES {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }
        
        // Extract nonce and encrypted data
        let nonce_bytes = &data[..secretbox::NONCEBYTES];
        let encrypted_data = &data[secretbox::NONCEBYTES..];
        
        // Convert nonce bytes to a Nonce
        let nonce = secretbox::Nonce::from_slice(nonce_bytes)
            .ok_or_else(|| anyhow::anyhow!("Invalid nonce"))?;
        
        // Get the key
        let key = self.get_or_create_key(user_id)?;
        
        // Decrypt the data
        let decrypted = secretbox::open(encrypted_data, &nonce, &key)
            .map_err(|_| anyhow::anyhow!("Decryption failed"))?;
        
        Ok(decrypted)
    }
    
    /// Rotate a user's key
    pub async fn rotate_key(&self, user_id: &str) -> Result<()> {
        // In a real implementation, this would involve fetching all encrypted data,
        // decrypting it with the old key, generating a new key, and re-encrypting
        // with the new key.
        let new_key = secretbox::gen_key();
        
        {
            let mut keys = self.user_keys.write().unwrap();
            keys.insert(user_id.to_string(), new_key);
        }
        
        Ok(())
    }
}
