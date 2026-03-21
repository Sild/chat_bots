use serde::{Deserialize, Serialize};

use crate::application;

#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    pub chat_id: i64,
    pub init_data: String,
}

#[derive(Debug, Deserialize)]
pub struct AddSpendRequest {
    pub chat_id: i64,
    pub init_data: String,
    pub total: String,
    pub mode: String,
    pub payer_user_id: i64,
    pub splits: Vec<SplitValueRequest>,
}

#[derive(Debug, Deserialize)]
pub struct ReportRequest {
    pub chat_id: i64,
    pub init_data: String,
}

#[derive(Debug, Deserialize)]
pub struct SplitValueRequest {
    pub user_id: i64,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub chat: ChatResponse,
    pub participant: ParticipantResponse,
    pub session: SessionResponse,
    pub participants: Vec<ParticipantResponse>,
    pub spends_count: i64,
    pub balances: Vec<BalanceResponse>,
    pub transfers: Vec<TransferResponse>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub chat_id: i64,
    pub chat_type: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParticipantResponse {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: i64,
    pub started_at: String,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub user_id: i64,
    pub display_name: String,
    pub net_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub from_name: String,
    pub to_name: String,
    pub amount_cents: i64,
}

impl From<application::Snapshot> for SnapshotResponse {
    fn from(value: application::Snapshot) -> Self {
        Self {
            chat: ChatResponse {
                chat_id: value.chat.chat_id,
                chat_type: value.chat.chat_type,
                title: value.chat.title,
            },
            participant: ParticipantResponse {
                user_id: value.participant.user_id,
                username: value.participant.username,
                first_name: value.participant.first_name,
                last_name: value.participant.last_name,
                display_name: value.participant.display_name,
            },
            session: SessionResponse {
                id: value.session.id,
                started_at: value.session.started_at,
            },
            participants: value
                .participants
                .into_iter()
                .map(|participant| ParticipantResponse {
                    user_id: participant.user_id,
                    username: participant.username,
                    first_name: participant.first_name,
                    last_name: participant.last_name,
                    display_name: participant.display_name,
                })
                .collect(),
            spends_count: value.spends_count,
            balances: value
                .balances
                .into_iter()
                .map(|balance| BalanceResponse {
                    user_id: balance.user_id,
                    display_name: balance.display_name,
                    net_cents: balance.net_cents,
                })
                .collect(),
            transfers: value
                .transfers
                .into_iter()
                .map(|transfer| TransferResponse {
                    from_user_id: transfer.from_user_id,
                    to_user_id: transfer.to_user_id,
                    from_name: transfer.from_name,
                    to_name: transfer.to_name,
                    amount_cents: transfer.amount_cents,
                })
                .collect(),
        }
    }
}
