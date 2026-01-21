mod new_group_handler;
mod server;
mod app_state;
mod bot;

use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
};
use tracing_subscriber::EnvFilter;
use crate::app_state::AppState;
use crate::server::run_server;

#[tokio::main]
async fn main() -> Result<()> {
    // logging
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let storage_path = PathBuf::from("data/storage.csv");
    let state = Arc::new(AppState::new(storage_path).await?);
    let token = dotenvy::var("SPLIT_SMART_BOT_TOKEN").expect("SPLIT_SMART_BOT_TOKEN not set");
    let tg_bot = bot::TGBot::new(state.clone(), &token)?;


    run_server(state.clone(), tg_bot).await?;
    Ok(())
}
