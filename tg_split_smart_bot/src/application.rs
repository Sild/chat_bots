mod add_spend;
mod bootstrap;
mod report;
mod reset;
mod snapshot;
mod types;

use crate::error::{AppError, AppResult};
use crate::infra::db::Database;
use crate::infra::telegram_auth::{self, ValidatedInitData};

#[derive(Clone)]
pub struct SplitSmartApplication {
    db: Database,
    bot_token: String,
}

impl SplitSmartApplication {
    pub fn new(db: Database, bot_token: String) -> Self {
        Self { db, bot_token }
    }

    pub fn authenticate_chat_request(
        &self,
        chat_id: i64,
        init_data: &str,
    ) -> AppResult<types::AuthenticatedChatRequest> {
        let validated = telegram_auth::validate_init_data(init_data, &self.bot_token)?;
        self.build_authenticated_chat_request(chat_id, validated)
    }

    pub async fn ensure_registered_member(&self, chat_id: i64, user_id: i64) -> AppResult<()> {
        self.db
            .get_participant(chat_id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Forbidden("participant is not registered for this chat".to_string())
            })?;
        Ok(())
    }

    pub async fn upsert_chat_metadata(
        &self,
        chat_id: i64,
        chat_type: &str,
        title: Option<&str>,
    ) -> AppResult<()> {
        self.db.upsert_chat(chat_id, chat_type, title).await?;
        Ok(())
    }

    fn build_authenticated_chat_request(
        &self,
        requested_chat_id: i64,
        validated: ValidatedInitData,
    ) -> AppResult<types::AuthenticatedChatRequest> {
        if validated.chat.id != requested_chat_id {
            return Err(AppError::Auth(
                "chat_id does not match signed Telegram chat context".to_string(),
            ));
        }

        Ok(types::AuthenticatedChatRequest {
            chat_id: validated.chat.id,
            chat_type: validated.chat.chat_type,
            chat_title: validated.chat.title,
            user_id: validated.user.id,
            username: validated.user.username,
            first_name: validated.user.first_name,
            last_name: validated.user.last_name,
        })
    }
}

pub use types::{
    AddSpendCommand, AddSpendResult, AuthenticatedChatRequest, BootstrapResult, Snapshot,
    SnapshotBalance, SnapshotChat, SnapshotParticipant, SnapshotSession, SnapshotTransfer,
    SpendModeDistribution, SplitInput, spend_distribution_text,
};
