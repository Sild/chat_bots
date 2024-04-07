use std::collections::HashSet;
use slack_cmd_core;
use anyhow::Result;
use slack_cmd_core::ALL_CHANNELS;

pub fn config_env_var(name: &str) -> std::result::Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let oauth_token = config_env_var("SLACK_CMD_OAUTH_TOKEN")?;
    let socket_token = config_env_var("SLACK_CMD_SOCKET_TOKEN")?;
    slack_cmd_core::run(
        oauth_token.as_str(),
        socket_token.as_str(),
        vec![
            slack_cmd_core::handlers::JiraHandler::new("123", "345", HashSet::from([ALL_CHANNELS.to_string()])),
         ],
    )
    .await?;
    Ok(())
}
