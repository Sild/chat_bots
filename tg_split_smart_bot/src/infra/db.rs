use chrono::Utc;
use sqlx::{SqlitePool, Transaction, Sqlite};

use crate::error::{AppError, AppResult};

// For compile-time SQL checks, enable the `sqlx-offline` feature and run `cargo sqlx prepare`.
#[cfg(feature = "sqlx-offline")]
#[allow(dead_code)]
fn _compile_time_sql_checks() {
    let _ = sqlx::query!("SELECT id, chat_id FROM sessions WHERE id = 1");
}

#[derive(Debug, Clone)]
pub struct ParticipantRecord {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub registered_at: String,
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: i64,
    pub chat_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpendRow {
    pub id: i64,
    pub payer_user_id: i64,
    pub total_cents: i64,
}

#[derive(Debug, Clone)]
pub struct AllocationRow {
    pub spend_id: i64,
    pub participant_user_id: i64,
    pub share_cents: i64,
}

#[derive(Debug, Clone)]
pub struct NewSpend {
    pub session_id: i64,
    pub created_at: String,
    pub creator_user_id: i64,
    pub payer_user_id: i64,
    pub total_cents: i64,
    pub mode: String,
}

pub async fn ensure_chat(
    pool: &SqlitePool,
    chat_id: i64,
    chat_type: &str,
    title: Option<&str>,
) -> AppResult<()> {
    let created_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO chats (chat_id, chat_type, title, created_at) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(chat_id) DO UPDATE SET chat_type = excluded.chat_type, title = excluded.title",
    )
    .bind(chat_id)
    .bind(chat_type)
    .bind(title)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn ensure_active_session(pool: &SqlitePool, chat_id: i64) -> AppResult<SessionRecord> {
    if let Some(row) = sqlx::query_as::<_, (i64, i64, String, Option<String>)>(
        "SELECT id, chat_id, started_at, ended_at FROM sessions WHERE chat_id = ?1 AND ended_at IS NULL ORDER BY id DESC LIMIT 1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await? {
        return Ok(SessionRecord {
            id: row.0,
            chat_id: row.1,
            started_at: row.2,
            ended_at: row.3,
        });
    }

    let started_at = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO sessions (chat_id, started_at, ended_at) VALUES (?1, ?2, NULL)",
    )
    .bind(chat_id)
    .bind(&started_at)
    .execute(pool)
    .await?;

    Ok(SessionRecord {
        id: result.last_insert_rowid(),
        chat_id,
        started_at,
        ended_at: None,
    })
}

pub async fn end_session(pool: &SqlitePool, session_id: i64) -> AppResult<()> {
    let ended_at = Utc::now().to_rfc3339();
    sqlx::query("UPDATE sessions SET ended_at = ?1 WHERE id = ?2")
        .bind(ended_at)
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_participant(
    pool: &SqlitePool,
    chat_id: i64,
    user_id: i64,
) -> AppResult<Option<ParticipantRecord>> {
    let row = sqlx::query_as::<_, (i64, i64, i64, Option<String>, Option<String>, Option<String>, String)>(
        "SELECT id, chat_id, user_id, username, first_name, last_name, registered_at FROM participants WHERE chat_id = ?1 AND user_id = ?2",
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| ParticipantRecord {
        id: r.0,
        chat_id: r.1,
        user_id: r.2,
        username: r.3,
        first_name: r.4,
        last_name: r.5,
        registered_at: r.6,
    }))
}

pub async fn upsert_participant(
    pool: &SqlitePool,
    chat_id: i64,
    user_id: i64,
    username: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
) -> AppResult<ParticipantRecord> {
    let registered_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO participants (chat_id, user_id, username, first_name, last_name, registered_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(chat_id, user_id) DO UPDATE SET username = excluded.username, first_name = excluded.first_name, last_name = excluded.last_name",
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(username)
    .bind(first_name)
    .bind(last_name)
    .bind(&registered_at)
    .execute(pool)
    .await?;

    get_participant(pool, chat_id, user_id)
        .await?
        .ok_or_else(|| AppError::Db(sqlx::Error::RowNotFound))
}

pub async fn list_participants(pool: &SqlitePool, chat_id: i64) -> AppResult<Vec<ParticipantRecord>> {
    let rows = sqlx::query_as::<_, (i64, i64, i64, Option<String>, Option<String>, Option<String>, String)>(
        "SELECT id, chat_id, user_id, username, first_name, last_name, registered_at FROM participants WHERE chat_id = ?1 ORDER BY user_id ASC",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ParticipantRecord {
            id: r.0,
            chat_id: r.1,
            user_id: r.2,
            username: r.3,
            first_name: r.4,
            last_name: r.5,
            registered_at: r.6,
        })
        .collect())
}

pub async fn count_spends(pool: &SqlitePool, session_id: i64) -> AppResult<i64> {
    let row = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(1) FROM spends WHERE session_id = ?1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn list_spends(pool: &SqlitePool, session_id: i64) -> AppResult<Vec<SpendRow>> {
    let rows = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, payer_user_id, total_cents FROM spends WHERE session_id = ?1 ORDER BY id ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SpendRow {
            id: r.0,
            payer_user_id: r.1,
            total_cents: r.2,
        })
        .collect())
}

pub async fn list_allocations(pool: &SqlitePool, session_id: i64) -> AppResult<Vec<AllocationRow>> {
    let rows = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT a.spend_id, a.participant_user_id, a.share_cents FROM allocations a \
         INNER JOIN spends s ON s.id = a.spend_id WHERE s.session_id = ?1 ORDER BY a.id ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| AllocationRow {
            spend_id: r.0,
            participant_user_id: r.1,
            share_cents: r.2,
        })
        .collect())
}

pub async fn insert_spend_with_allocations(
    pool: &SqlitePool,
    spend: NewSpend,
    allocations: &[AllocationRow],
) -> AppResult<i64> {
    let mut tx: Transaction<'_, Sqlite> = pool.begin().await?;
    let result = sqlx::query(
        "INSERT INTO spends (session_id, created_at, creator_user_id, payer_user_id, total_cents, mode) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(spend.session_id)
    .bind(spend.created_at)
    .bind(spend.creator_user_id)
    .bind(spend.payer_user_id)
    .bind(spend.total_cents)
    .bind(spend.mode)
    .execute(&mut *tx)
    .await?;
    let spend_id = result.last_insert_rowid();

    for alloc in allocations {
        sqlx::query(
            "INSERT INTO allocations (spend_id, participant_user_id, share_cents) VALUES (?1, ?2, ?3)",
        )
        .bind(spend_id)
        .bind(alloc.participant_user_id)
        .bind(alloc.share_cents)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(spend_id)
}

pub fn format_display_name(username: &Option<String>, first_name: &Option<String>, last_name: &Option<String>) -> String {
    if let Some(handle) = username {
        if !handle.is_empty() {
            return format!("@{}", handle);
        }
    }
    let mut name = String::new();
    if let Some(first) = first_name {
        if !first.is_empty() {
            name.push_str(first);
        }
    }
    if let Some(last) = last_name {
        if !last.is_empty() {
            if !name.is_empty() {
                name.push(' ');
            }
            name.push_str(last);
        }
    }
    if name.is_empty() {
        "Unknown".to_string()
    } else {
        name
    }
}
