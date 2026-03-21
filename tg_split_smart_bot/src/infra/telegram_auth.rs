use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use url::form_urlencoded;

use crate::error::{AppError, AppResult};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidatedInitData {
    pub user: TelegramUser,
    pub chat: TelegramChat,
}

pub fn validate_init_data(init_data: &str, bot_token: &str) -> AppResult<ValidatedInitData> {
    if init_data.trim().is_empty() {
        return Err(AppError::Auth("missing Telegram init data".to_string()));
    }

    let params: BTreeMap<String, String> = form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();
    let provided_hash = params
        .get("hash")
        .ok_or_else(|| AppError::Auth("missing init data hash".to_string()))?;

    let data_check_string = params
        .iter()
        .filter(|(key, _)| *key != "hash")
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut secret_key_mac = HmacSha256::new_from_slice(b"WebAppData")
        .map_err(|_| AppError::Auth("failed to initialize Telegram auth".to_string()))?;
    secret_key_mac.update(bot_token.as_bytes());
    let secret_key = secret_key_mac.finalize().into_bytes();

    let mut data_mac = HmacSha256::new_from_slice(&secret_key)
        .map_err(|_| AppError::Auth("failed to validate Telegram auth".to_string()))?;
    data_mac.update(data_check_string.as_bytes());
    let expected_hash = hex::encode(data_mac.finalize().into_bytes());

    if !constant_time_eq(provided_hash.as_bytes(), expected_hash.as_bytes()) {
        return Err(AppError::Auth(
            "invalid Telegram init data signature".to_string(),
        ));
    }

    let user: TelegramUser = serde_json::from_str(
        params
            .get("user")
            .ok_or_else(|| AppError::Auth("missing user in init data".to_string()))?,
    )
    .map_err(|_| AppError::Auth("invalid user payload in init data".to_string()))?;

    let chat: TelegramChat = serde_json::from_str(
        params
            .get("chat")
            .ok_or_else(|| AppError::Auth("missing chat in init data".to_string()))?,
    )
    .map_err(|_| AppError::Auth("invalid chat payload in init data".to_string()))?;

    Ok(ValidatedInitData { user, chat })
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}
