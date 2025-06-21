use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use super::models::{User, CreateUserInput, Credentials};

/// Repository for user-related operations
pub struct IdentityRepository {
    pool: PgPool,
}

impl IdentityRepository {
    /// Create a new identity repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Create a new user
    pub async fn create_user(&self, input: CreateUserInput) -> Result<User> {
        // Hash the password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(input.password.as_bytes(), &salt)?
            .to_string();
        
        // Insert the user
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, display_name, password_hash)
            VALUES ($1, $2, $3)
            RETURNING 
                id, email, display_name, created_at, updated_at
            "#,
            input.email,
            input.display_name,
            password_hash,
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    /// Get a user by ID
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, email, display_name, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    /// Get a user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, email, display_name, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    /// Authenticate a user with credentials
    pub async fn authenticate(&self, credentials: Credentials) -> Result<Option<User>> {
        // Get the user and password hash
        let result = sqlx::query!(
            r#"
            SELECT 
                id, email, display_name, created_at, updated_at, password_hash
            FROM users
            WHERE email = $1
            "#,
            credentials.email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        let Some(user_data) = result else {
            return Ok(None);
        };
        
        // Verify the password
        let parsed_hash = PasswordHash::new(&user_data.password_hash)?;
        let password_matches = Argon2::default()
            .verify_password(credentials.password.as_bytes(), &parsed_hash)
            .is_ok();
        
        if password_matches {
            Ok(Some(User {
                id: user_data.id,
                email: user_data.email,
                display_name: user_data.display_name,
                created_at: user_data.created_at,
                updated_at: user_data.updated_at,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Update a user's profile
    pub async fn update_user(
        &self,
        id: Uuid,
        display_name: Option<String>,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET 
                display_name = COALESCE($2, display_name),
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, email, display_name, created_at, updated_at
            "#,
            id,
            display_name,
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }
}
