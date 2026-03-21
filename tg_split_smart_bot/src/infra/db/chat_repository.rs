use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::models::Chat;
use crate::error::{AppError, AppResult};

pub(super) async fn upsert_chat(
    pool: &SqlitePool,
    chat_id: i64,
    chat_type: &str,
    title: Option<&str>,
) -> AppResult<Chat> {
    sqlx::query(
        "INSERT INTO chats (chat_id, chat_type, title, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(chat_id) DO UPDATE
         SET chat_type = excluded.chat_type, title = excluded.title",
    )
    .bind(chat_id)
    .bind(chat_type)
    .bind(title)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    get_chat(pool, chat_id).await
}

pub(super) async fn get_chat(pool: &SqlitePool, chat_id: i64) -> AppResult<Chat> {
    let row = sqlx::query_as::<_, (i64, String, Option<String>, String)>(
        "SELECT chat_id, chat_type, title, created_at FROM chats WHERE chat_id = ?1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?;

    row.map(|(chat_id, chat_type, title, created_at)| Chat {
        chat_id,
        chat_type,
        title,
        created_at,
    })
    .ok_or_else(|| AppError::NotFound("chat not found".to_string()))
}
