use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

use super::{
    models::{User, CreateUserInput, Credentials, AuthToken},
    repository::IdentityRepository,
};

/// Claims for JWT tokens
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject (user ID)
    sub: String,
    /// Expiration time
    exp: i64,
    /// Issued at
    iat: i64,
}

/// Service for managing users and authentication
pub struct IdentityService {
    repository: IdentityRepository,
    jwt_secret: String,
}

impl IdentityService {
    /// Create a new identity service
    pub fn new(pool: PgPool) -> Arc<Self> {
        // In a real app, this would be loaded from environment variables
        let jwt_secret = "supersecret123".to_string();
        
        Arc::new(Self {
            repository: IdentityRepository::new(pool),
            jwt_secret,
        })
    }
    
    /// Create a new user
    pub async fn create_user(&self, input: CreateUserInput) -> Result<User> {
        self.repository.create_user(input).await
    }
    
    /// Get a user by ID
    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        self.repository.get_user_by_id(id).await
    }
    
    /// Get a user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.repository.get_user_by_email(email).await
    }
    
    /// Authenticate a user with credentials
    pub async fn authenticate(&self, credentials: Credentials) -> Result<Option<AuthToken>> {
        let user = match self.repository.authenticate(credentials).await? {
            Some(u) => u,
            None => return Ok(None),
        };
        
        // Generate a JWT token
        let expiration = Utc::now() + Duration::hours(24);
        
        let claims = Claims {
            sub: user.id.to_string(),
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;
        
        Ok(Some(AuthToken {
            token,
            expires_at: expiration,
            user,
        }))
    }
    
    /// Validate a JWT token
    pub async fn validate_token(&self, token: &str) -> Result<Option<Uuid>> {
        let validation = Validation::default();
        
        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(data) => data,
            Err(_) => return Ok(None),
        };
        
        let user_id = Uuid::parse_str(&token_data.claims.sub)?;
        
        Ok(Some(user_id))
    }
    
    /// Update a user's profile
    pub async fn update_user(
        &self,
        id: Uuid,
        display_name: Option<String>,
    ) -> Result<Option<User>> {
        self.repository.update_user(id, display_name).await
    }
}
