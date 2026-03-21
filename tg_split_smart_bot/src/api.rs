mod dto;
mod handlers;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::app_state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(handlers::health))
        .route("/tg", get(handlers::app_page))
        .route("/app", get(handlers::app_page))
        .route("/api/bootstrap", post(handlers::bootstrap))
        .route("/api/spends", post(handlers::add_spend))
        .route("/api/report", post(handlers::report))
        .route("/api/reset", post(handlers::reset))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
