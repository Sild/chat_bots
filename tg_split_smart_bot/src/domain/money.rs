use std::str::FromStr;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::domain::models::Cents;
use crate::error::{AppError, AppResult};

pub fn parse_money_to_cents(input: &str) -> AppResult<Cents> {
    let decimal = parse_decimal(input, "amount")?;
    if decimal <= Decimal::ZERO {
        return Err(AppError::Validation(
            "amount must be greater than zero".to_string(),
        ));
    }
    let cents = (decimal * Decimal::new(100, 0))
        .to_i64()
        .ok_or_else(|| AppError::Validation("amount is too large".to_string()))?;
    Ok(cents)
}

pub fn parse_absolute_share_to_cents(input: &str) -> AppResult<Cents> {
    let decimal = parse_decimal(input, "share")?;
    if decimal.is_sign_negative() {
        return Err(AppError::Validation(
            "share amounts cannot be negative".to_string(),
        ));
    }
    let cents = (decimal * Decimal::new(100, 0))
        .to_i64()
        .ok_or_else(|| AppError::Validation("share is too large".to_string()))?;
    Ok(cents)
}

pub fn parse_percent(input: &str) -> AppResult<Decimal> {
    let decimal = parse_decimal(input, "percent")?;
    if decimal.is_sign_negative() {
        return Err(AppError::Validation(
            "percent values cannot be negative".to_string(),
        ));
    }
    Ok(decimal)
}

pub fn format_cents(cents: Cents) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let absolute = cents.abs();
    format!("{sign}{}.{:02}", absolute / 100, absolute % 100)
}

fn parse_decimal(input: &str, field_name: &str) -> AppResult<Decimal> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{field_name} is required")));
    }

    let decimal = Decimal::from_str(trimmed)
        .map_err(|_| AppError::Validation(format!("invalid {field_name}")))?;
    if decimal.scale() > 2 {
        return Err(AppError::Validation(format!(
            "{field_name} must have at most 2 decimal places"
        )));
    }
    Ok(decimal)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{format_cents, parse_absolute_share_to_cents, parse_money_to_cents};

    #[test]
    fn test_parse_money_to_cents_accepts_two_decimals() {
        assert_eq!(parse_money_to_cents("12.34").unwrap(), 1_234);
    }

    #[test]
    fn test_parse_money_to_cents_rejects_zero() {
        let error = parse_money_to_cents("0").unwrap_err();
        assert_eq!(error.to_string(), "amount must be greater than zero");
    }

    #[test]
    fn test_parse_money_to_cents_rejects_more_than_two_decimals() {
        let error = parse_money_to_cents("1.999").unwrap_err();
        assert_eq!(
            error.to_string(),
            "amount must have at most 2 decimal places"
        );
    }

    #[test]
    fn test_parse_absolute_share_to_cents_allows_zero() {
        assert_eq!(parse_absolute_share_to_cents("0").unwrap(), 0);
    }

    #[test]
    fn test_format_cents_renders_sign_and_scale() {
        assert_eq!(format_cents(1_205), "12.05");
        assert_eq!(format_cents(-305), "-3.05");
    }
}
