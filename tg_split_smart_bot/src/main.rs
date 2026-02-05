mod api;
mod app_state;
mod application;
mod bot;
mod config;
mod domain;
mod error;
mod infra;

use std::sync::Arc;

use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::app_state::AppState;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = Arc::new(Config::from_env()?);
    if let Some(parent) = std::path::Path::new(&config.sqlite_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db_url = format!("sqlite://{}", config.sqlite_path);
    let db = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let bot = teloxide::Bot::new(config.bot_token.clone());
    let state = Arc::new(AppState::new(db, bot.clone(), config.clone()));

    let app = api::router(state.clone()).layer(tower_http::cors::CorsLayer::permissive());
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    info!(%addr, "Starting HTTP server");

    let server_task = tokio::spawn(async move {
        axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
        Ok::<(), anyhow::Error>(())
    });

    let bot_task = tokio::spawn(async move { bot::run(bot, state).await });

    let (server_res, bot_res) = tokio::try_join!(server_task, bot_task)?;
    server_res?;
    bot_res?;
    Ok(())
}
