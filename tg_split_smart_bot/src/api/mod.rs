use std::sync::Arc;

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::{Json, Router};
use axum::routing::{get, post};
use teloxide::payloads::SendMessageSetters;
use teloxide::types::ParseMode;
use tracing::info;

use crate::app_state::AppState;
use crate::application::{
    AddSpendRequest, BootstrapRequest, ReportRequest, BootstrapResponse,
};
use crate::error::{AppError, AppResult};
use crate::infra::telegram_auth;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(app_index))
        .route("/app", get(app_index))
        .route("/api/bootstrap", post(handle_bootstrap))
        .route("/api/spends", post(handle_add_spend))
        .route("/api/report", post(handle_report))
        .route("/api/reset", post(handle_reset))
        .with_state(state)
}

async fn app_index() -> impl IntoResponse {
    Html(include_str!("../../frontend/index.html"))
}

async fn handle_bootstrap(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BootstrapRequest>,
) -> Result<Json<BootstrapResponse>, AppError> {
    let chat_id = req.chat_id;
    let bot = state.bot.clone();
    let response = crate::application::bootstrap(
        &state.db,
        &state.config.bot_token,
        chat_id,
        &req.init_data,
        move |text| {
            let bot = bot.clone();
            Box::pin(async move {
                bot.send_message(chat_id, text).await?;
                Ok(())
            })
        },
    )
    .await?;

    Ok(Json(response))
}

async fn handle_add_spend(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSpendRequest>,
) -> Result<Json<BootstrapResponse>, AppError> {
    let chat_id = req.chat_id;
    let bot = state.bot.clone();
    let response = crate::application::add_spend(
        &state.db,
        &state.config.bot_token,
        req,
        move |text| {
            let bot = bot.clone();
            Box::pin(async move {
                bot.send_message(chat_id, text)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
                Ok(())
            })
        },
    )
    .await?;

    Ok(Json(response))
}

async fn handle_report(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReportRequest>,
) -> Result<Json<BootstrapResponse>, AppError> {
    let response = crate::application::report(
        &state.db,
        &state.config.bot_token,
        req.chat_id,
        &req.init_data,
    )
    .await?;
    Ok(Json(response))
}

async fn handle_reset(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReportRequest>,
) -> Result<Json<BootstrapResponse>, AppError> {
    let init = telegram_auth::validate_init_data(&req.init_data, &state.config.bot_token)?;
    ensure_admin(&state, req.chat_id, init.user.id).await?;

    let chat_id = req.chat_id;
    let bot = state.bot.clone();
    let response = crate::application::reset_session(
        &state.db,
        req.chat_id,
        &req.init_data,
        &state.config.bot_token,
        move |text| {
            let bot = bot.clone();
            Box::pin(async move {
                bot.send_message(chat_id, text)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;
                Ok(())
            })
        },
    )
    .await?;
    Ok(Json(response))
}

async fn ensure_admin(state: &AppState, chat_id: i64, user_id: i64) -> AppResult<()> {
    if chat_id > 0 {
        return Ok(());
    }
    let member = state.bot.get_chat_member(chat_id, user_id).await?;
    let status = member.status();
    let allowed = matches!(
        status,
        teloxide::types::ChatMemberStatus::Administrator
            | teloxide::types::ChatMemberStatus::Creator
    );
    if !allowed {
        info!(chat_id, user_id, "reset denied: not admin");
        return Err(AppError::Auth("admin permissions required".to_string()));
    }
    Ok(())
}
