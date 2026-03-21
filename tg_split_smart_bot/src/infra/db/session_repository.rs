use chrono::Utc;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::domain::models::Session;
use crate::error::AppResult;

pub(super) async fn ensure_active_session(pool: &SqlitePool, chat_id: i64) -> AppResult<Session> {
    if let Some(session) = get_active_session(pool, chat_id).await? {
        return Ok(session);
    }

    let mut transaction = pool.begin().await?;
    let session = create_session(&mut transaction, chat_id).await?;
    transaction.commit().await?;
    Ok(session)
}

pub(super) async fn get_session(pool: &SqlitePool, session_id: i64) -> AppResult<Option<Session>> {
    let row = sqlx::query_as::<_, (i64, i64, String, Option<String>)>(
        "SELECT id, chat_id, started_at, ended_at FROM sessions WHERE id = ?1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, chat_id, started_at, ended_at)| Session {
        id,
        chat_id,
        started_at,
        ended_at,
    }))
}

async fn get_active_session(pool: &SqlitePool, chat_id: i64) -> AppResult<Option<Session>> {
    let row = sqlx::query_as::<_, (i64, i64, String, Option<String>)>(
        "SELECT id, chat_id, started_at, ended_at
         FROM sessions
         WHERE chat_id = ?1 AND ended_at IS NULL
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, chat_id, started_at, ended_at)| Session {
        id,
        chat_id,
        started_at,
        ended_at,
    }))
}

pub(super) async fn close_active_session(
    transaction: &mut Transaction<'_, Sqlite>,
    chat_id: i64,
    ended_at: &str,
) -> AppResult<()> {
    sqlx::query("UPDATE sessions SET ended_at = ?1 WHERE chat_id = ?2 AND ended_at IS NULL")
        .bind(ended_at)
        .bind(chat_id)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub(super) async fn create_session(
    transaction: &mut Transaction<'_, Sqlite>,
    chat_id: i64,
) -> AppResult<Session> {
    let started_at = Utc::now().to_rfc3339();
    let result =
        sqlx::query("INSERT INTO sessions (chat_id, started_at, ended_at) VALUES (?1, ?2, NULL)")
            .bind(chat_id)
            .bind(&started_at)
            .execute(&mut **transaction)
            .await?;

    Ok(Session {
        id: result.last_insert_rowid(),
        chat_id,
        started_at,
        ended_at: None,
    })
}
