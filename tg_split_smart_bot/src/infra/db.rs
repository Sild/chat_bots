mod chat_repository;
mod participant_repository;
mod session_repository;
mod snapshot_repository;
mod spend_repository;

use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::models::{Allocation, Chat, Participant, Session, SpendLedgerEntry, SpendMode};
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct RegistrationResult {
    pub participant: Participant,
    pub is_new: bool,
}

#[derive(Debug, Clone)]
pub struct SnapshotSource {
    pub chat: Chat,
    pub session: Session,
    pub participants: Vec<Participant>,
    pub spends_count: i64,
    pub ledger: Vec<SpendLedgerEntry>,
}

#[derive(Debug, Clone)]
pub struct CreateSpendInput {
    pub session_id: i64,
    pub creator_user_id: i64,
    pub payer_user_id: i64,
    pub total_cents: i64,
    pub mode: SpendMode,
    pub allocations: Vec<Allocation>,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert_chat(
        &self,
        chat_id: i64,
        chat_type: &str,
        title: Option<&str>,
    ) -> AppResult<Chat> {
        chat_repository::upsert_chat(&self.pool, chat_id, chat_type, title).await
    }

    pub async fn get_chat(&self, chat_id: i64) -> AppResult<Chat> {
        chat_repository::get_chat(&self.pool, chat_id).await
    }

    pub async fn ensure_active_session(&self, chat_id: i64) -> AppResult<Session> {
        session_repository::ensure_active_session(&self.pool, chat_id).await
    }

    pub async fn get_participant(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> AppResult<Option<Participant>> {
        participant_repository::get_participant(&self.pool, chat_id, user_id).await
    }

    pub async fn register_participant(
        &self,
        chat_id: i64,
        user_id: i64,
        username: Option<&str>,
        first_name: &str,
        last_name: Option<&str>,
    ) -> AppResult<RegistrationResult> {
        let existing: Option<Participant> =
            participant_repository::get_participant(&self.pool, chat_id, user_id).await?;
        let participant = participant_repository::upsert_participant(
            &self.pool, chat_id, user_id, username, first_name, last_name,
        )
        .await?;
        Ok(RegistrationResult {
            participant,
            is_new: existing.is_none(),
        })
    }

    pub async fn list_participants(&self, chat_id: i64) -> AppResult<Vec<Participant>> {
        participant_repository::list_participants(&self.pool, chat_id).await
    }

    pub async fn create_spend(&self, input: CreateSpendInput) -> AppResult<i64> {
        spend_repository::insert_spend_with_allocations(&self.pool, input).await
    }

    pub async fn load_snapshot_source(
        &self,
        chat_id: i64,
        session_id: i64,
    ) -> AppResult<SnapshotSource> {
        let chat = self.get_chat(chat_id).await?;
        let session: Session = session_repository::get_session(&self.pool, session_id)
            .await?
            .ok_or_else(|| AppError::NotFound("session not found".to_string()))?;
        let participants = participant_repository::list_participants(&self.pool, chat_id).await?;
        let spends_count = snapshot_repository::count_spends(&self.pool, session_id).await?;
        let ledger = snapshot_repository::list_ledger(&self.pool, session_id).await?;

        Ok(SnapshotSource {
            chat,
            session,
            participants,
            spends_count,
            ledger,
        })
    }

    pub async fn reset_session(&self, chat_id: i64) -> AppResult<Session> {
        let mut transaction = self.pool.begin().await?;
        let ended_at = Utc::now().to_rfc3339();
        session_repository::close_active_session(&mut transaction, chat_id, &ended_at).await?;
        let new_session = session_repository::create_session(&mut transaction, chat_id).await?;
        transaction.commit().await?;
        Ok(new_session)
    }
}
