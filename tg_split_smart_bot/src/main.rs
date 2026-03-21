use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use reqwest::Proxy;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::{error, info, warn};
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
    let bot = build_bot_client(&config)?;
    let telegram = TelegramGateway::new(bot, config.app_base_url.clone());
    let application = SplitSmartApplication::new(database.clone(), config.bot_token.clone());
    let state = Arc::new(AppState::new(
        application,
        telegram,
        config.telegram_bot_username.clone(),
    ));

    info!(
        public_base_url = %config.public_base_url,
        telegram_bot_username = %config.telegram_bot_username,
        telegram_proxy_enabled = config.telegram_proxy_url.is_some(),
        "starting SplitSmart",
    );

    let app = tg_split_smart_bot::api::router(state.clone());
    let bind_addr = config.bind_addr;
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("failed to bind http listener on {bind_addr}"))?;
    info!(%bind_addr, "http server listening");

    tokio::spawn(async move {
        loop {
            let state = state.clone();
            let result = tokio::spawn(async move {
                let bot_runner = SplitSmartBot::new(state);
                bot_runner.run().await
            })
            .await;

            match result {
                Ok(Ok(())) => warn!("bot runner exited unexpectedly; restarting"),
                Ok(Err(error)) => error!(?error, "bot runner failed; restarting"),
                Err(error) => error!(?error, "bot runner panicked; restarting"),
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    axum::serve(listener, app)
        .await
        .context("http server terminated")?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn build_bot_client(config: &Config) -> anyhow::Result<teloxide::Bot> {
    let builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(17))
        .tcp_nodelay(true);
    let builder = if let Some(proxy_url) = &config.telegram_proxy_url {
        builder.proxy(Proxy::all(proxy_url).with_context(|| {
            format!("failed to parse TELEGRAM_PROXY_URL/TELOXIDE_PROXY: {proxy_url}")
        })?)
    } else {
        builder
    };
    let client = builder
        .build()
        .context("failed to build reqwest client for Telegram bot")?;

    Ok(teloxide::Bot::with_client(config.bot_token.clone(), client))
}

fn ensure_sqlite_parent(sqlite_path: &str) -> anyhow::Result<()> {
    if let Some(parent) = Path::new(sqlite_path).parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create sqlite parent directory for {sqlite_path}")
        })?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(sqlite_path)
        .with_context(|| format!("failed to create sqlite database file at {sqlite_path}"))?;
    Ok(())
}
