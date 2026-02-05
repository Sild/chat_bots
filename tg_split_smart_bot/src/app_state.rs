use std::sync::Arc;

use sqlx::SqlitePool;
use teloxide::Bot;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub bot: Bot,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: SqlitePool, bot: Bot, config: Arc<Config>) -> Self {
        Self { db, bot, config }
    }
}
