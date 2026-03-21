use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::infra::db::CreateSpendInput;

pub(super) async fn insert_spend_with_allocations(
    pool: &SqlitePool,
    input: CreateSpendInput,
) -> AppResult<i64> {
    let mut transaction = pool.begin().await?;
    let spend_result = sqlx::query(
        "INSERT INTO spends (session_id, created_at, creator_user_id, payer_user_id, total_cents, mode)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(input.session_id)
    .bind(Utc::now().to_rfc3339())
    .bind(input.creator_user_id)
    .bind(input.payer_user_id)
    .bind(input.total_cents)
    .bind(input.mode.as_db_value())
    .execute(&mut *transaction)
    .await?;
    let spend_id = spend_result.last_insert_rowid();

    for allocation in input.allocations {
        sqlx::query(
            "INSERT INTO allocations (spend_id, participant_user_id, share_cents)
             VALUES (?1, ?2, ?3)",
        )
        .bind(spend_id)
        .bind(allocation.participant_user_id)
        .bind(allocation.share_cents)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(spend_id)
}
