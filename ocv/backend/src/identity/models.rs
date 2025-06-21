use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a user in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier
    pub id: Uuid,
    
    /// Email address
    pub email: String,
    
    /// Display name
    pub display_name: String,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new user
#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserInput {
    /// Email address
    pub email: String,
    
    /// Display name
    pub display_name: String,
    
    /// Password (will be hashed before storage)
    pub password: String,
}

/// Authentication credentials
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// Email address
    pub email: String,
    
    /// Password
    pub password: String,
}

/// Authentication token
#[derive(Debug, Clone, Serialize)]
pub struct AuthToken {
    /// JWT token
    pub token: String,
    
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
    
    /// User information
    pub user: User,
}
