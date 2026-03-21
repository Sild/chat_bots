use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::domain::models::{Allocation, SpendLedgerEntry};
use crate::error::AppResult;

pub(super) async fn count_spends(pool: &SqlitePool, session_id: i64) -> AppResult<i64> {
    let (count,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(1) FROM spends WHERE session_id = ?1")
        .bind(session_id)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub(super) async fn list_ledger(
    pool: &SqlitePool,
    session_id: i64,
) -> AppResult<Vec<SpendLedgerEntry>> {
    let spend_rows = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, payer_user_id, total_cents
         FROM spends
         WHERE session_id = ?1
         ORDER BY id ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    let allocation_rows = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT a.spend_id, a.participant_user_id, a.share_cents
         FROM allocations a
         INNER JOIN spends s ON s.id = a.spend_id
         WHERE s.session_id = ?1
         ORDER BY a.spend_id ASC, a.participant_user_id ASC",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    let mut allocations_by_spend: HashMap<i64, Vec<Allocation>> = HashMap::new();
    for (spend_id, participant_user_id, share_cents) in allocation_rows {
        allocations_by_spend
            .entry(spend_id)
            .or_default()
            .push(Allocation {
                participant_user_id,
                share_cents,
            });
    }

    Ok(spend_rows
        .into_iter()
        .map(|(spend_id, payer_user_id, total_cents)| SpendLedgerEntry {
            payer_user_id,
            total_cents,
            allocations: allocations_by_spend.remove(&spend_id).unwrap_or_default(),
        })
        .collect())
}
