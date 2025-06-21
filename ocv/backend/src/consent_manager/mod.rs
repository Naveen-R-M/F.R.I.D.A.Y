pub mod models;
pub mod repository;
pub mod service;
pub mod graphql;

// Re-export key types
pub use models::{AccessGrant, GrantAccessInput};
pub use service::ConsentManager;
