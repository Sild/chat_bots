use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub bot_token: String,
    pub public_base_url: String,
    pub webapp_url: String,
    pub sqlite_path: String,
    pub telegram_bot_username: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bot_token = env::var("BOT_TOKEN")?;
        let public_base_url = env::var("PUBLIC_BASE_URL")?;
        let sqlite_path = env::var("SQLITE_PATH")?;
        let telegram_bot_username = env::var("TELEGRAM_BOT_USERNAME")?;
        let webapp_url = env::var("WEBAPP_URL").unwrap_or_else(|_| format!("{}/app", public_base_url));
        Ok(Self {
            bot_token,
            public_base_url,
            webapp_url,
            sqlite_path,
            telegram_bot_username,
        })
    }
}
