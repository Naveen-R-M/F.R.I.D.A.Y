use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde_json::Value;

/// Simple policy engine using Open Policy Agent concepts
#[derive(Clone)]
pub struct PolicyEngine {
    // In a real implementation, this would use OPA WASM modules
    // This is a simplified version for demonstration purposes
    policies: Arc<RwLock<HashMap<String, String>>>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Arc<Self> {
        let mut policies = HashMap::new();
        
        // Add default policies
        policies.insert(
            "consent".to_string(),
            r#"
            package consent

            # Allow users to grant access to their own data
            allow {
                input.action == "grant"
                input.user != null
            }
            
            # Allow users to revoke their own grants
            allow {
                input.action == "revoke"
                input.user != null
                input.grant_id != null
            }
            
            # Default deny
            default allow = false
            "#.to_string(),
        );
        
        Arc::new(Self {
            policies: Arc::new(RwLock::new(policies)),
        })
    }
    
    /// Add or update a policy
    pub async fn update_policy(&self, name: &str, policy: &str) -> Result<()> {
        let mut policies = self.policies.write().unwrap();
        policies.insert(name.to_string(), policy.to_string());
        Ok(())
    }
    
    /// Evaluate a policy against input data
    pub async fn evaluate(&self, policy_name: &str, input: Value) -> Result<bool> {
        let policies = self.policies.read().unwrap();
        
        // Check if the policy exists
        let policy = match policies.get(policy_name) {
            Some(p) => p,
            None => return Err(anyhow::anyhow!("Policy not found: {}", policy_name)),
        };
        
        // In a real implementation, this would compile and evaluate the policy
        // with OPA WASM. This is a simplified version that makes basic decisions.
        
        // For now, we'll just make some simple decisions based on the input
        if policy_name == "consent" {
            let action = input.get("action").and_then(|a| a.as_str()).unwrap_or("");
            
            match action {
                "grant" => {
                    // Allow granting as long as there's a user
                    input.get("user").is_some()
                },
                "revoke" => {
                    // Allow revoking as long as there's a user and grant_id
                    input.get("user").is_some() && input.get("grant_id").is_some()
                },
                _ => false,
            }
        } else {
            // Default deny for unknown policies
            false
        }
    }
}
