use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use crate::app_state::AppState;
use crate::bot::TGBot;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Entry {
    user_id: i64,
    user_name: String,
    number: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddRequest {
    user_id: i64,
    user_name: String,
    number: i64,
}



fn read_entries(path: &PathBuf) -> anyhow::Result<Vec<Entry>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut rdr = ReaderBuilder::new().from_path(path)?;
    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let record: Entry = result?;
        entries.push(record);
    }
    Ok(entries)
}

fn write_entry(path: &PathBuf, entry: &Entry) -> anyhow::Result<()> {
    let file_exists = path.exists();
    let mut wtr = WriterBuilder::new().from_path(path)?;
    if !file_exists {
        let _ = wtr.write_record(["user_id", "user_name", "number"])?;
    }
    wtr.serialize(entry)?;
    wtr.flush()?;
    Ok(())
}

async fn axum_add(State(state): State<Arc<AppState>>, Json(req): Json<AddRequest>) -> impl IntoResponse {
    let entry = Entry { user_id: req.user_id, user_name: req.user_name, number: req.number };
    match write_entry(&state.storage_path, &entry) {
        Ok(_) => (StatusCode::OK, Json(entry)).into_response(),
        Err(e) => {
            warn!(error=?e, "Failed to write entry");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}

async fn axum_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match read_entries(&state.storage_path) {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            warn!(error=?e, "Failed to read entries");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}

#[derive(Serialize)]
struct UsersResponse {
    users: Vec<(i64, String)>,
}

async fn axum_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match read_entries(&state.storage_path) {
        Ok(entries) => {
            let mut uniq = Vec::<(i64, String)>::new();
            for e in entries {
                if !uniq.iter().any(|(id, _)| *id == e.user_id) {
                    uniq.push((e.user_id, e.user_name));
                }
            }
            Json(UsersResponse { users: uniq }).into_response()
        }
        Err(e) => {
            warn!(error=?e, "Failed to read users");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response()
        }
    }
}

async fn axum_index() -> impl IntoResponse {
    Html(include_str!("../frontend/index.html")).into_response()
}

pub async fn run_server(state: Arc<AppState>, _bot: TGBot) -> anyhow::Result<()> {
    let cors = tower_http::cors::CorsLayer::very_permissive();
    let app = Router::new()
        .route("/", get(axum_index))
        .route("/api/add", post(axum_add))
        .route("/api/list", get(axum_list))
        .route("/api/users", get(axum_users))
        .with_state(state)
        .layer(cors);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    info!(%addr, "Starting axum server");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}