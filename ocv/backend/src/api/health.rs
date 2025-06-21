use actix_web::{get, web, HttpResponse};
use serde::Serialize;

use crate::api::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    database: String,
}

/// Health check endpoint
#[get("/health")]
pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    // Check database connection
    let db_status = match state.pool.acquire().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status.to_string(),
    };

    HttpResponse::Ok().json(response)
}
