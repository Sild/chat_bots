use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::models::Participant;
use crate::error::{AppError, AppResult};

pub(super) async fn get_participant(
    pool: &SqlitePool,
    chat_id: i64,
    user_id: i64,
) -> AppResult<Option<Participant>> {
    let row = sqlx::query_as::<_, (i64, Option<String>, String, Option<String>, String)>(
        "SELECT user_id, username, first_name, last_name, registered_at
         FROM participants
         WHERE chat_id = ?1 AND user_id = ?2",
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(user_id, username, first_name, last_name, registered_at)| Participant {
            user_id,
            username,
            first_name,
            last_name,
            registered_at,
        },
    ))
}

pub(super) async fn upsert_participant(
    pool: &SqlitePool,
    chat_id: i64,
    user_id: i64,
    username: Option<&str>,
    first_name: &str,
    last_name: Option<&str>,
) -> AppResult<Participant> {
    sqlx::query(
        "INSERT INTO participants (chat_id, user_id, username, first_name, last_name, registered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(chat_id, user_id) DO UPDATE
         SET username = excluded.username, first_name = excluded.first_name, last_name = excluded.last_name",
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(username)
    .bind(first_name)
    .bind(last_name)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    let participant: Option<Participant> = get_participant(pool, chat_id, user_id).await?;
    participant.ok_or_else(|| AppError::NotFound("participant not found".to_string()))
}

pub(super) async fn list_participants(
    pool: &SqlitePool,
    chat_id: i64,
) -> AppResult<Vec<Participant>> {
    let rows = sqlx::query_as::<_, (i64, Option<String>, String, Option<String>, String)>(
        "SELECT user_id, username, first_name, last_name, registered_at
         FROM participants
         WHERE chat_id = ?1
         ORDER BY user_id ASC",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(user_id, username, first_name, last_name, registered_at)| Participant {
                user_id,
                username,
                first_name,
                last_name,
                registered_at,
            },
        )
        .collect())
}
