use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::form_urlencoded;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
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
pub struct InitData {
    pub user: TelegramUser,
    pub chat: Option<TelegramChat>,
    pub auth_date: Option<i64>,
}

type HmacSha256 = Hmac<Sha256>;

pub fn validate_init_data(init_data: &str, bot_token: &str) -> AppResult<InitData> {
    let pairs: BTreeMap<String, String> = form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();
    let hash = pairs
        .get("hash")
        .ok_or_else(|| AppError::Auth("missing init data hash".to_string()))?
        .to_string();

    let mut data_check_parts = Vec::new();
    for (k, v) in &pairs {
        if k == "hash" {
            continue;
        }
        data_check_parts.push(format!("{}={}", k, v));
    }
    data_check_parts.sort();
    let data_check_string = data_check_parts.join("\n");

    let secret_key = Sha256::digest(bot_token.as_bytes());
    let mut mac = HmacSha256::new_from_slice(&secret_key)
        .map_err(|_| AppError::Auth("invalid bot token".to_string()))?;
    mac.update(data_check_string.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    if !constant_time_eq::constant_time_eq(expected.as_bytes(), hash.as_bytes()) {
        return Err(AppError::Auth("invalid init data signature".to_string()));
    }

    let user_json = pairs
        .get("user")
        .ok_or_else(|| AppError::Auth("missing user in init data".to_string()))?;
    let user: TelegramUser = serde_json::from_str(user_json)
        .map_err(|_| AppError::Auth("invalid user in init data".to_string()))?;

    let chat = match pairs.get("chat") {
        Some(chat_json) => Some(
            serde_json::from_str(chat_json)
                .map_err(|_| AppError::Auth("invalid chat in init data".to_string()))?,
        ),
        None => None,
    };

    let auth_date = pairs
        .get("auth_date")
        .and_then(|v| v.parse::<i64>().ok());

    Ok(InitData {
        user,
        chat,
        auth_date,
    })
}

mod constant_time_eq {
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut result = 0u8;
        for i in 0..a.len() {
            result |= a[i] ^ b[i];
        }
        result == 0
    }
}
