use crate::error::{AppError, AppResult};

pub type Cents = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chat {
    pub chat_id: i64,
    pub chat_type: String,
    pub title: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participant {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub registered_at: String,
}

impl Participant {
    pub fn display_name(&self) -> String {
        match self.last_name.as_deref() {
            Some(last_name) if !last_name.is_empty() => {
                format!("{} {}", self.first_name, last_name)
            }
            _ => self.first_name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: i64,
    pub chat_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendMode {
    Abs,
    Percent,
}

impl SpendMode {
    pub fn parse(input: &str) -> AppResult<Self> {
        match input.trim().to_ascii_uppercase().as_str() {
            "ABS" => Ok(Self::Abs),
            "PERCENT" => Ok(Self::Percent),
            _ => Err(AppError::Validation(
                "mode must be ABS or PERCENT".to_string(),
            )),
        }
    }

    pub fn as_db_value(self) -> &'static str {
        match self {
            Self::Abs => "ABS",
            Self::Percent => "PERCENT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    pub participant_user_id: i64,
    pub share_cents: Cents,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendLedgerEntry {
    pub payer_user_id: i64,
    pub total_cents: Cents,
    pub allocations: Vec<Allocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balance {
    pub user_id: i64,
    pub net_cents: Cents,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub amount_cents: Cents,
}
