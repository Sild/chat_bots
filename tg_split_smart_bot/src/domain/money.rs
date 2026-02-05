use std::str::FromStr;

use rust_decimal::prelude::ToPrimitive;

use crate::error::{AppError, AppResult};

pub fn parse_amount_to_cents(input: &str) -> AppResult<i64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("amount is required".to_string()));
    }
    let dec = rust_decimal::Decimal::from_str(trimmed)
        .map_err(|_| AppError::Validation("invalid amount".to_string()))?;
    if dec.is_sign_negative() {
        return Err(AppError::Validation("amount must be positive".to_string()));
    }
    if dec.scale() > 2 {
        return Err(AppError::Validation("amount must have at most 2 decimals".to_string()));
    }
    let cents = (dec * rust_decimal::Decimal::new(100, 0))
        .round()
        .to_i64()
        .ok_or_else(|| AppError::Validation("amount too large".to_string()))?;
    Ok(cents)
}

pub fn parse_percent(input: &str) -> AppResult<rust_decimal::Decimal> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("percent is required".to_string()));
    }
    let dec = rust_decimal::Decimal::from_str(trimmed)
        .map_err(|_| AppError::Validation("invalid percent".to_string()))?;
    if dec.is_sign_negative() {
        return Err(AppError::Validation("percent must be positive".to_string()));
    }
    if dec.scale() > 2 {
        return Err(AppError::Validation("percent must have at most 2 decimals".to_string()));
    }
    Ok(dec)
}

pub fn cents_to_string(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    format!("{}{}.{}", sign, abs / 100, format!("{:02}", abs % 100))
}
