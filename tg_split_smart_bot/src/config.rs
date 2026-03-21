use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub bot_token: String,
    pub public_base_url: String,
    pub app_base_url: String,
    pub sqlite_path: String,
    pub telegram_bot_username: String,
    pub telegram_proxy_url: Option<String>,
    pub bind_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bot_token = env::var("SPLIT_SMART_BOT_TOKEN")
            .or_else(|_| env::var("BOT_TOKEN"))
            .context("missing SPLIT_SMART_BOT_TOKEN (or BOT_TOKEN fallback)")?;
        let public_base_url = env::var("PUBLIC_BASE_URL").context("missing PUBLIC_BASE_URL")?;
        let sqlite_path = env::var("SQLITE_PATH").context("missing SQLITE_PATH")?;
        let telegram_bot_username =
            env::var("TELEGRAM_BOT_USERNAME").context("missing TELEGRAM_BOT_USERNAME")?;
        let telegram_proxy_url = env::var("TELEGRAM_PROXY_URL")
            .ok()
            .or_else(|| env::var("TELOXIDE_PROXY").ok());
        let public_base_url = public_base_url.trim_end_matches('/').to_string();
        let app_base_url = format!("{public_base_url}/tg");

        Ok(Self {
            bot_token,
            public_base_url,
            app_base_url,
            sqlite_path,
            telegram_bot_username,
            telegram_proxy_url,
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080),
        })
    }

    pub fn sqlite_url(&self) -> String {
        format!("sqlite://{}", self.sqlite_path)
    }
}
