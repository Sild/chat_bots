use std::collections::HashMap;

use crate::application::SplitSmartApplication;
use crate::application::types::{
    Snapshot, SnapshotBalance, SnapshotChat, SnapshotParticipant, SnapshotSession, SnapshotTransfer,
};
use crate::domain::settlement::{compute_balances, compute_transfers};
use crate::error::{AppError, AppResult};

impl SplitSmartApplication {
    pub(crate) async fn build_snapshot(
        &self,
        chat_id: i64,
        session_id: i64,
        actor_user_id: i64,
    ) -> AppResult<Snapshot> {
        let source = self.db.load_snapshot_source(chat_id, session_id).await?;
        let actor_user_id = source
            .participants
            .iter()
            .find(|participant| participant.user_id == actor_user_id)
            .map(|participant| participant.user_id)
            .ok_or_else(|| AppError::NotFound("participant not found".to_string()))?;

        Ok(build_snapshot_from_source(source, Some(actor_user_id)))
    }

    pub(crate) async fn build_snapshot_without_actor(
        &self,
        chat_id: i64,
        session_id: i64,
    ) -> AppResult<Snapshot> {
        let source = self.db.load_snapshot_source(chat_id, session_id).await?;
        Ok(build_snapshot_from_source(source, None))
    }
}

fn build_snapshot_from_source(
    source: crate::infra::db::SnapshotSource,
    actor_user_id: Option<i64>,
) -> Snapshot {
    let participants: Vec<SnapshotParticipant> = source
        .participants
        .iter()
        .map(SnapshotParticipant::from_domain)
        .collect();
    let participant_ids = source
        .participants
        .iter()
        .map(|participant| participant.user_id)
        .collect::<Vec<_>>();
    let balances = compute_balances(&participant_ids, &source.ledger);
    let transfers = compute_transfers(&balances);
    let names_by_id: HashMap<i64, String> = participants
        .iter()
        .map(|participant| (participant.user_id, participant.display_name.clone()))
        .collect();
    let current_participant = actor_user_id
        .and_then(|user_id| {
            participants
                .iter()
                .find(|participant| participant.user_id == user_id)
        })
        .cloned()
        .unwrap_or_else(|| {
            participants
                .first()
                .cloned()
                .unwrap_or(SnapshotParticipant {
                    user_id: 0,
                    username: None,
                    first_name: String::new(),
                    last_name: None,
                    display_name: String::new(),
                })
        });

    Snapshot {
        chat: SnapshotChat {
            chat_id: source.chat.chat_id,
            chat_type: source.chat.chat_type,
            title: source.chat.title,
        },
        participant: current_participant,
        session: SnapshotSession::from_domain(&source.session),
        participants: participants.clone(),
        spends_count: source.spends_count,
        balances: balances
            .into_iter()
            .map(|balance| SnapshotBalance {
                user_id: balance.user_id,
                display_name: names_by_id
                    .get(&balance.user_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                net_cents: balance.net_cents,
            })
            .collect(),
        transfers: transfers
            .into_iter()
            .map(|transfer| SnapshotTransfer {
                from_user_id: transfer.from_user_id,
                to_user_id: transfer.to_user_id,
                from_name: names_by_id
                    .get(&transfer.from_user_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                to_name: names_by_id
                    .get(&transfer.to_user_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string()),
                amount_cents: transfer.amount_cents,
            })
            .collect(),
    }
}
