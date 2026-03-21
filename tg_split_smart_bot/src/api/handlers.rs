use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use tracing::warn;

use crate::api::dto::{AddSpendRequest, BootstrapRequest, ReportRequest, SnapshotResponse};
use crate::app_state::AppState;
use crate::application::{AddSpendCommand, SplitInput};
use crate::error::AppError;

pub async fn app_page() -> impl IntoResponse {
    Html(include_str!("../../static/app.html"))
}

pub async fn bootstrap(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BootstrapRequest>,
) -> Result<Json<SnapshotResponse>, AppError> {
    let auth = state
        .application
        .authenticate_chat_request(request.chat_id, &request.init_data)?;
    let result = state.application.bootstrap(&auth).await?;
    if let Some(message) = result.registration_message.as_deref()
        && let Err(error) = state
            .telegram
            .send_registration_message(request.chat_id, message)
            .await
    {
        warn!(
            chat_id = request.chat_id,
            ?error,
            "failed to send registration message"
        );
    }

    Ok(Json(result.snapshot.into()))
}

pub async fn add_spend(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AddSpendRequest>,
) -> Result<Json<SnapshotResponse>, AppError> {
    let auth = state
        .application
        .authenticate_chat_request(request.chat_id, &request.init_data)?;
    let result = state
        .application
        .add_spend(AddSpendCommand {
            auth,
            total: request.total,
            mode: request.mode,
            payer_user_id: request.payer_user_id,
            splits: request
                .splits
                .into_iter()
                .map(|split| SplitInput {
                    user_id: split.user_id,
                    value: split.value,
                })
                .collect(),
        })
        .await?;

    if let Err(error) = state
        .telegram
        .send_spend_message(request.chat_id, &result.spend_message)
        .await
    {
        warn!(
            chat_id = request.chat_id,
            ?error,
            "failed to send spend message"
        );
    }

    Ok(Json(result.snapshot.into()))
}

pub async fn report(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReportRequest>,
) -> Result<Json<SnapshotResponse>, AppError> {
    let auth = state
        .application
        .authenticate_chat_request(request.chat_id, &request.init_data)?;
    let snapshot = state.application.report_for_member(&auth).await?;
    Ok(Json(snapshot.into()))
}

pub async fn reset(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReportRequest>,
) -> Result<Json<SnapshotResponse>, AppError> {
    let auth = state
        .application
        .authenticate_chat_request(request.chat_id, &request.init_data)?;
    state
        .application
        .ensure_registered_member(auth.chat_id, auth.user_id)
        .await?;
    state
        .telegram
        .ensure_admin(auth.chat_id, auth.user_id)
        .await?;

    let report = state.application.render_report(auth.chat_id).await?;
    state
        .telegram
        .send_report_message(auth.chat_id, &report)
        .await?;

    let snapshot = state.application.reset_for_member(&auth).await?;
    Ok(Json(snapshot.into()))
}
