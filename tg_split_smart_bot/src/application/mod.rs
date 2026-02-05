use std::collections::{HashMap, HashSet};

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::domain::markdown::escape_markdown_v2;
use crate::domain::money::{cents_to_string, parse_amount_to_cents, parse_percent};
use crate::domain::settlement::{compute_balances, compute_transfers, Allocation, Spend};
use crate::error::{AppError, AppResult};
use crate::infra::db::{
    AllocationRow, NewSpend, ParticipantRecord, SessionRecord,
};
use crate::infra::{db, telegram_auth};

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
    pub splits: Vec<SplitInput>,
}

#[derive(Debug, Deserialize)]
pub struct ReportRequest {
    pub chat_id: i64,
    pub init_data: String,
}

#[derive(Debug, Deserialize)]
pub struct SplitInput {
    pub user_id: i64,
    pub value: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatInfo {
    pub chat_id: i64,
    pub chat_type: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ParticipantInfo {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SessionInfo {
    pub id: i64,
    pub started_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct BalanceInfo {
    pub user_id: i64,
    pub display_name: String,
    pub net_cents: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TransferInfo {
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub from_name: String,
    pub to_name: String,
    pub amount_cents: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct BootstrapResponse {
    pub chat: ChatInfo,
    pub participant: ParticipantInfo,
    pub session: SessionInfo,
    pub spends_count: i64,
    pub balances: Vec<BalanceInfo>,
    pub transfers: Vec<TransferInfo>,
    pub participants: Vec<ParticipantInfo>,
}

pub async fn bootstrap(
    pool: &sqlx::SqlitePool,
    bot_token: &str,
    chat_id: i64,
    init_data: &str,
    notify_new_participant: impl Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + Send>>,
) -> AppResult<BootstrapResponse> {
    let init = telegram_auth::validate_init_data(init_data, bot_token)?;
    if let Some(chat) = &init.chat {
        if chat.id != chat_id {
            return Err(AppError::Validation("chat_id does not match init data".to_string()));
        }
        db::ensure_chat(pool, chat.id, &chat.chat_type, chat.title.as_deref()).await?;
    } else {
        warn!(chat_id, "init data missing chat context; allowing request");
        db::ensure_chat(pool, chat_id, "unknown", None).await?;
    }

    let existing = db::get_participant(pool, chat_id, init.user.id).await?;
    let participant = db::upsert_participant(
        pool,
        chat_id,
        init.user.id,
        init.user.username.as_deref(),
        init.user.first_name.as_deref(),
        init.user.last_name.as_deref(),
    )
    .await?;

    if existing.is_none() {
        let display = db::format_display_name(&participant.username, &participant.first_name, &participant.last_name);
        info!(chat_id, user_id = participant.user_id, "new participant registered");
        notify_new_participant(format!("{} registered", display)).await?;
    }

    let session = db::ensure_active_session(pool, chat_id).await?;
    snapshot(pool, chat_id, &participant, &session).await
}

pub async fn add_spend(
    pool: &sqlx::SqlitePool,
    bot_token: &str,
    req: AddSpendRequest,
    notify_spend: impl Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + Send>>,
) -> AppResult<BootstrapResponse> {
    let init = telegram_auth::validate_init_data(&req.init_data, bot_token)?;
    if let Some(chat) = &init.chat {
        if chat.id != req.chat_id {
            return Err(AppError::Validation("chat_id does not match init data".to_string()));
        }
        db::ensure_chat(pool, chat.id, &chat.chat_type, chat.title.as_deref()).await?;
    } else {
        warn!(chat_id = req.chat_id, "init data missing chat context; allowing request");
        db::ensure_chat(pool, req.chat_id, "unknown", None).await?;
    }

    let participants = db::list_participants(pool, req.chat_id).await?;
    let participant_map: HashMap<i64, ParticipantRecord> = participants
        .iter()
        .cloned()
        .map(|p| (p.user_id, p))
        .collect();

    if !participant_map.contains_key(&init.user.id) {
        return Err(AppError::Validation(
            "participant must register before adding spends".to_string(),
        ));
    }

    if !participant_map.contains_key(&req.payer_user_id) {
        return Err(AppError::Validation("payer is not a registered participant".to_string()));
    }

    let unique: HashSet<i64> = req.splits.iter().map(|s| s.user_id).collect();
    if unique.len() != req.splits.len() {
        return Err(AppError::Validation("duplicate users in splits".to_string()));
    }

    if req.splits.len() != participants.len() {
        return Err(AppError::Validation(
            "splits must include all participants".to_string(),
        ));
    }

    for split in &req.splits {
        if !participant_map.contains_key(&split.user_id) {
            return Err(AppError::Validation("split includes unknown participant".to_string()));
        }
    }

    let total_cents = parse_amount_to_cents(&req.total)?;
    let mode = req.mode.to_uppercase();
    if mode != "ABS" && mode != "PERCENT" {
        return Err(AppError::Validation("mode must be ABS or PERCENT".to_string()));
    }

    let allocations = if mode == "ABS" {
        build_abs_allocations(total_cents, &req.splits)?
    } else {
        build_percent_allocations(total_cents, &req.splits)?
    };

    let session = db::ensure_active_session(pool, req.chat_id).await?;
    let spend = NewSpend {
        session_id: session.id,
        created_at: Utc::now().to_rfc3339(),
        creator_user_id: init.user.id,
        payer_user_id: req.payer_user_id,
        total_cents,
        mode: mode.clone(),
    };
    db::insert_spend_with_allocations(pool, spend, &allocations).await?;

    let creator = participant_map.get(&init.user.id).cloned();
    let payer = participant_map.get(&req.payer_user_id).cloned();
    let distribution = render_distribution(&allocations, &participant_map);
    let creator_name = creator
        .as_ref()
        .map(|p| db::format_display_name(&p.username, &p.first_name, &p.last_name))
        .unwrap_or_else(|| "Someone".to_string());
    let payer_name = payer
        .as_ref()
        .map(|p| db::format_display_name(&p.username, &p.first_name, &p.last_name))
        .unwrap_or_else(|| "Someone".to_string());
    let header = format!(
        "{} added new spend. Paid by {}. Distribution: ",
        creator_name, payer_name
    );
    let message = format!(
        "{}||{}||",
        escape_markdown_v2(&header),
        escape_markdown_v2(&distribution)
    );
    notify_spend(message).await?;

    let participant = db::get_participant(pool, req.chat_id, init.user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("participant not found".to_string()))?;
    snapshot(pool, req.chat_id, &participant, &session).await
}

pub async fn report(
    pool: &sqlx::SqlitePool,
    bot_token: &str,
    chat_id: i64,
    init_data: &str,
) -> AppResult<BootstrapResponse> {
    let init = telegram_auth::validate_init_data(init_data, bot_token)?;
    if let Some(chat) = &init.chat {
        if chat.id != chat_id {
            return Err(AppError::Validation("chat_id does not match init data".to_string()));
        }
        db::ensure_chat(pool, chat.id, &chat.chat_type, chat.title.as_deref()).await?;
    } else {
        warn!(chat_id, "init data missing chat context; allowing request");
        db::ensure_chat(pool, chat_id, "unknown", None).await?;
    }

    let participant = db::get_participant(pool, chat_id, init.user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("participant not found".to_string()))?;
    let session = db::ensure_active_session(pool, chat_id).await?;
    snapshot(pool, chat_id, &participant, &session).await
}

pub async fn reset_session(
    pool: &sqlx::SqlitePool,
    chat_id: i64,
    init_data: &str,
    bot_token: &str,
    send_report: impl Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + Send>>,
) -> AppResult<BootstrapResponse> {
    let init = telegram_auth::validate_init_data(init_data, bot_token)?;
    if let Some(chat) = &init.chat {
        if chat.id != chat_id {
            return Err(AppError::Validation("chat_id does not match init data".to_string()));
        }
        db::ensure_chat(pool, chat.id, &chat.chat_type, chat.title.as_deref()).await?;
    } else {
        warn!(chat_id, "init data missing chat context; allowing request");
        db::ensure_chat(pool, chat_id, "unknown", None).await?;
    }

    let participant = db::get_participant(pool, chat_id, init.user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("participant not found".to_string()))?;
    let session = db::ensure_active_session(pool, chat_id).await?;
    let report_text = render_report(pool, chat_id, &session).await?;
    send_report(report_text).await?;
    db::end_session(pool, session.id).await?;
    let new_session = db::ensure_active_session(pool, chat_id).await?;
    snapshot(pool, chat_id, &participant, &new_session).await
}

pub async fn render_report(pool: &sqlx::SqlitePool, chat_id: i64, session: &SessionRecord) -> AppResult<String> {
    let participants = db::list_participants(pool, chat_id).await?;
    let snapshot = snapshot_from_parts(pool, chat_id, session, &participants).await?;

    let mut lines = Vec::new();
    lines.push(format!("SplitSmart report for chat {}", chat_id));
    lines.push(format!("Participants: {}", participants.len()));
    for p in &snapshot.participants {
        lines.push(format!("Participant: {}", p.display_name));
    }
    lines.push(format!("Spends: {}", snapshot.spends_count));
    lines.push("Balances:".to_string());
    for b in &snapshot.balances {
        lines.push(format!("Balance {} {}", b.display_name, cents_to_string(b.net_cents)));
    }
    lines.push("Transfers:".to_string());
    if snapshot.transfers.is_empty() {
        lines.push("No transfers needed".to_string());
    } else {
        for t in &snapshot.transfers {
            lines.push(format!(
                "Transfer {} -> {} {}",
                t.from_name,
                t.to_name,
                cents_to_string(t.amount_cents)
            ));
        }
    }

    Ok(escape_markdown_v2(&lines.join("\n")))
}

async fn snapshot(
    pool: &sqlx::SqlitePool,
    chat_id: i64,
    participant: &ParticipantRecord,
    session: &SessionRecord,
) -> AppResult<BootstrapResponse> {
    let participants = db::list_participants(pool, chat_id).await?;
    snapshot_from_parts(pool, chat_id, session, &participants).await.map(|mut snapshot| {
        snapshot.participant = ParticipantInfo {
            user_id: participant.user_id,
            username: participant.username.clone(),
            first_name: participant.first_name.clone(),
            last_name: participant.last_name.clone(),
            display_name: db::format_display_name(&participant.username, &participant.first_name, &participant.last_name),
        };
        snapshot
    })
}

async fn snapshot_from_parts(
    pool: &sqlx::SqlitePool,
    chat_id: i64,
    session: &SessionRecord,
    participants: &[ParticipantRecord],
) -> AppResult<BootstrapResponse> {
    let spends_count = db::count_spends(pool, session.id).await?;
    let spends = db::list_spends(pool, session.id).await?;
    let allocations = db::list_allocations(pool, session.id).await?;

    let spends_map: HashMap<i64, Vec<Allocation>> = {
        let mut map = HashMap::new();
        for alloc in allocations {
            map.entry(alloc.spend_id)
                .or_insert_with(Vec::new)
                .push(Allocation {
                    participant_user_id: alloc.participant_user_id,
                    share_cents: alloc.share_cents,
                });
        }
        map
    };

    let spend_models: Vec<Spend> = spends
        .into_iter()
        .map(|s| Spend {
            payer_user_id: s.payer_user_id,
            total_cents: s.total_cents,
            allocations: spends_map.get(&s.id).cloned().unwrap_or_default(),
        })
        .collect();

    let balance_map = compute_balances(&spend_models);
    let transfers = compute_transfers(&balance_map);

    let participants_info: Vec<ParticipantInfo> = participants
        .iter()
        .map(|p| ParticipantInfo {
            user_id: p.user_id,
            username: p.username.clone(),
            first_name: p.first_name.clone(),
            last_name: p.last_name.clone(),
            display_name: db::format_display_name(&p.username, &p.first_name, &p.last_name),
        })
        .collect();

    let mut balance_info = Vec::new();
    for p in participants {
        let net = balance_map.get(&p.user_id).copied().unwrap_or(0);
        balance_info.push(BalanceInfo {
            user_id: p.user_id,
            display_name: db::format_display_name(&p.username, &p.first_name, &p.last_name),
            net_cents: net,
        });
    }

    let name_map: HashMap<i64, String> = participants
        .iter()
        .map(|p| (p.user_id, db::format_display_name(&p.username, &p.first_name, &p.last_name)))
        .collect();

    let transfer_info = transfers
        .into_iter()
        .map(|t| TransferInfo {
            from_user_id: t.from_user_id,
            to_user_id: t.to_user_id,
            from_name: name_map
                .get(&t.from_user_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
            to_name: name_map
                .get(&t.to_user_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
            amount_cents: t.amount_cents,
        })
        .collect();

    let chat = ChatInfo {
        chat_id,
        chat_type: init_chat_type(pool, chat_id).await?,
        title: init_chat_title(pool, chat_id).await?,
    };

    Ok(BootstrapResponse {
        chat,
        participant: ParticipantInfo {
            user_id: 0,
            username: None,
            first_name: None,
            last_name: None,
            display_name: "".to_string(),
        },
        session: SessionInfo {
            id: session.id,
            started_at: session.started_at.clone(),
        },
        spends_count,
        balances: balance_info,
        transfers: transfer_info,
        participants: participants_info,
    })
}

fn build_abs_allocations(total_cents: i64, splits: &[SplitInput]) -> AppResult<Vec<AllocationRow>> {
    if splits.is_empty() {
        return Err(AppError::Validation("splits cannot be empty".to_string()));
    }
    let mut allocations = Vec::new();
    let mut sum = 0i64;
    for split in splits {
        let cents = parse_amount_to_cents(&split.value)?;
        sum += cents;
        allocations.push(AllocationRow {
            spend_id: 0,
            participant_user_id: split.user_id,
            share_cents: cents,
        });
    }
    if sum != total_cents {
        return Err(AppError::Validation("split amounts must sum to total".to_string()));
    }
    Ok(allocations)
}

fn build_percent_allocations(total_cents: i64, splits: &[SplitInput]) -> AppResult<Vec<AllocationRow>> {
    if splits.is_empty() {
        return Err(AppError::Validation("splits cannot be empty".to_string()));
    }
    let mut parsed = Vec::new();
    let mut sum_pct = Decimal::new(0, 0);
    for split in splits {
        let pct = parse_percent(&split.value)?;
        sum_pct += pct;
        parsed.push((split.user_id, pct));
    }
    if sum_pct != Decimal::new(100, 0) {
        return Err(AppError::Validation("percent splits must sum to 100".to_string()));
    }

    let total_dec = Decimal::from_i64(total_cents)
        .ok_or_else(|| AppError::Validation("total too large".to_string()))?;
    let mut floors = Vec::new();
    let mut sum_floor = 0i64;
    for (user_id, pct) in &parsed {
        let raw = (total_dec * *pct) / Decimal::new(100, 0);
        let floor = raw.trunc();
        let floor_i64 = floor.to_i64().ok_or_else(|| AppError::Validation("allocation too large".to_string()))?;
        let remainder = raw - floor;
        floors.push((*user_id, floor_i64, remainder));
        sum_floor += floor_i64;
    }

    let mut allocations: Vec<AllocationRow> = floors
        .iter()
        .map(|(user_id, floor_i64, _)| AllocationRow {
            spend_id: 0,
            participant_user_id: *user_id,
            share_cents: *floor_i64,
        })
        .collect();

    let mut remainder = total_cents - sum_floor;
    if remainder != 0 {
        let mut sorted: Vec<(i64, Decimal)> = floors
            .into_iter()
            .map(|(user_id, _, rem)| (user_id, rem))
            .collect();
        if remainder > 0 {
            sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        } else {
            sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        }
        let mut idx = 0usize;
        while remainder != 0 {
            let user_id = sorted[idx % sorted.len()].0;
            for alloc in &mut allocations {
                if alloc.participant_user_id == user_id {
                    alloc.share_cents += if remainder > 0 { 1 } else { -1 };
                    remainder += if remainder > 0 { -1 } else { 1 };
                    break;
                }
            }
            idx += 1;
        }
    }

    Ok(allocations)
}

fn render_distribution(
    allocations: &[AllocationRow],
    participants: &HashMap<i64, ParticipantRecord>,
) -> String {
    let mut parts = Vec::new();
    for alloc in allocations {
        let name = participants
            .get(&alloc.participant_user_id)
            .map(|p| db::format_display_name(&p.username, &p.first_name, &p.last_name))
            .unwrap_or_else(|| "Unknown".to_string());
        parts.push(format!("{} {}", name, cents_to_string(alloc.share_cents)));
    }
    parts.join(", ")
}

async fn init_chat_type(pool: &sqlx::SqlitePool, chat_id: i64) -> AppResult<String> {
    let row = sqlx::query_as::<_, (String,)>("SELECT chat_type FROM chats WHERE chat_id = ?1")
        .bind(chat_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0).unwrap_or_else(|| "unknown".to_string()))
}

async fn init_chat_title(pool: &sqlx::SqlitePool, chat_id: i64) -> AppResult<Option<String>> {
    let row = sqlx::query_as::<_, (Option<String>,)>("SELECT title FROM chats WHERE chat_id = ?1")
        .bind(chat_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0).unwrap_or(None))
}
