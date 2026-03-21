use rust_decimal::Decimal;

use crate::domain::models::{Session, SpendMode};

#[derive(Debug, Clone)]
pub struct AuthenticatedChatRequest {
    pub chat_id: i64,
    pub chat_type: String,
    pub chat_title: Option<String>,
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SplitInput {
    pub user_id: i64,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct AddSpendCommand {
    pub auth: AuthenticatedChatRequest,
    pub total: String,
    pub mode: String,
    pub payer_user_id: i64,
    pub splits: Vec<SplitInput>,
}

#[derive(Debug, Clone)]
pub struct BootstrapResult {
    pub snapshot: Snapshot,
    pub registration_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AddSpendResult {
    pub snapshot: Snapshot,
    pub spend_message: String,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub chat: SnapshotChat,
    pub participant: SnapshotParticipant,
    pub session: SnapshotSession,
    pub participants: Vec<SnapshotParticipant>,
    pub spends_count: i64,
    pub balances: Vec<SnapshotBalance>,
    pub transfers: Vec<SnapshotTransfer>,
}

#[derive(Debug, Clone)]
pub struct SnapshotChat {
    pub chat_id: i64,
    pub chat_type: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SnapshotParticipant {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct SnapshotSession {
    pub id: i64,
    pub started_at: String,
}

#[derive(Debug, Clone)]
pub struct SnapshotBalance {
    pub user_id: i64,
    pub display_name: String,
    pub net_cents: i64,
}

#[derive(Debug, Clone)]
pub struct SnapshotTransfer {
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub from_name: String,
    pub to_name: String,
    pub amount_cents: i64,
}

#[derive(Debug, Clone)]
pub enum SpendModeDistribution {
    Abs(Vec<(i64, i64)>),
    Percent(Vec<(i64, Decimal)>),
}

pub fn spend_distribution_text(
    mode: SpendMode,
    distribution: &SpendModeDistribution,
    participants: &[SnapshotParticipant],
) -> String {
    let names_by_id = participants
        .iter()
        .map(|participant| (participant.user_id, participant.display_name.clone()))
        .collect::<std::collections::HashMap<_, _>>();

    match (mode, distribution) {
        (SpendMode::Abs, SpendModeDistribution::Abs(values)) => values
            .iter()
            .map(|(user_id, cents)| {
                format!(
                    "{} {}",
                    names_by_id
                        .get(user_id)
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    crate::domain::money::format_cents(*cents)
                )
            })
            .collect::<Vec<_>>()
            .join(", "),
        (SpendMode::Percent, SpendModeDistribution::Percent(values)) => values
            .iter()
            .map(|(user_id, percent)| {
                let rendered = if percent.fract().is_zero() {
                    percent.trunc().to_string()
                } else {
                    percent.normalize().to_string()
                };
                format!(
                    "{} {}%",
                    names_by_id
                        .get(user_id)
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string()),
                    rendered
                )
            })
            .collect::<Vec<_>>()
            .join(", "),
        _ => String::new(),
    }
}

impl SnapshotParticipant {
    pub fn from_domain(participant: &crate::domain::models::Participant) -> Self {
        Self {
            user_id: participant.user_id,
            username: participant.username.clone(),
            first_name: participant.first_name.clone(),
            last_name: participant.last_name.clone(),
            display_name: participant.display_name(),
        }
    }
}

impl SnapshotSession {
    pub fn from_domain(session: &Session) -> Self {
        Self {
            id: session.id,
            started_at: session.started_at.clone(),
        }
    }
}
