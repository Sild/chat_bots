use crate::handler::JiraHandler;
use crate::state::State;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

mod handler;
mod slack_cmd;
mod slack_helper;
mod state;

pub fn config_env_var(name: &str) -> std::result::Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let oauth_token = config_env_var("SLACK_CMD_OAUTH_TOKEN")?;
    let socket_token = config_env_var("SLACK_CMD_SOCKET_TOKEN")?;
    slack_cmd::run(
        oauth_token.as_str(),
        socket_token.as_str(),
        vec![Arc::new(RwLock::new(Box::new(JiraHandler::new("123", "345"))))],
    )
    .await?;
    Ok(())
}
