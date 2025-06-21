pub mod schema;
mod graphql;
mod health;

use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    consent_manager::service::ConsentManager,
    context_management::service::ContextService,
    encryption::service::EncryptionService,
    identity::service::IdentityService,
    policy_engine::service::PolicyEngine,
};

/// Application state shared across all routes
pub struct AppState {
    pub pool: PgPool,
    pub context_service: Arc<ContextService>,
    pub consent_manager: Arc<ConsentManager>,
    pub encryption_service: Arc<EncryptionService>,
    pub policy_engine: Arc<PolicyEngine>,
    pub identity_service: Arc<IdentityService>,
}

/// Configure all application routes and middleware
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(health::health_check)
            .service(graphql::graphql_handler)
            .service(graphql::graphql_playground)
    )
    .route("/", web::get().to(index));
}

/// Simple index handler
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Open Context Vault API")
}
