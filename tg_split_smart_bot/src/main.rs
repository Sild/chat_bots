use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::info;
use tracing_subscriber::EnvFilter;

use tg_split_smart_bot::app_state::AppState;
use tg_split_smart_bot::application::SplitSmartApplication;
use tg_split_smart_bot::bot::SplitSmartBot;
use tg_split_smart_bot::config::Config;
use tg_split_smart_bot::infra::db::Database;
use tg_split_smart_bot::infra::telegram::TelegramGateway;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Arc::new(Config::from_env()?);
    ensure_sqlite_parent(&config.sqlite_path)?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.sqlite_url())
        .await
        .context("failed to connect to sqlite")?;
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .context("failed to enable foreign keys")?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    let database = Database::new(pool);
    let bot = teloxide::Bot::new(config.bot_token.clone());
    let telegram = TelegramGateway::new(bot, config.app_base_url.clone());
    let application = SplitSmartApplication::new(database.clone(), config.bot_token.clone());
    let state = Arc::new(AppState::new(application, telegram));

    info!(
        public_base_url = %config.public_base_url,
        telegram_bot_username = %config.telegram_bot_username,
        "starting SplitSmart",
    );

    let app = tg_split_smart_bot::api::router(state.clone());
    let bind_addr = config.bind_addr;
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("failed to bind http listener on {bind_addr}"))?;
    info!(%bind_addr, "http server listening");

    let http_task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .context("http server terminated")
    });

    let bot_task = tokio::spawn(async move {
        let bot_runner = SplitSmartBot::new(state);
        bot_runner.run().await
    });

    let (http_result, bot_result) = tokio::try_join!(http_task, bot_task)?;
    http_result?;
    bot_result?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn ensure_sqlite_parent(sqlite_path: &str) -> anyhow::Result<()> {
    if let Some(parent) = Path::new(sqlite_path).parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create sqlite parent directory for {sqlite_path}")
        })?;
    }
    Ok(())
}
