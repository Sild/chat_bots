use std::collections::{HashMap, HashSet};

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::application::SplitSmartApplication;
use crate::application::types::{
    AddSpendCommand, AddSpendResult, SnapshotParticipant, SpendModeDistribution,
};
use crate::domain::models::{Allocation, SpendMode};
use crate::domain::money::{
    format_cents, parse_absolute_share_to_cents, parse_money_to_cents, parse_percent,
};
use crate::domain::telegram_markdown::escape_markdown_v2;
use crate::error::{AppError, AppResult};
use crate::infra::db::CreateSpendInput;

impl SplitSmartApplication {
    pub async fn add_spend(&self, command: AddSpendCommand) -> AppResult<AddSpendResult> {
        self.db
            .upsert_chat(
                command.auth.chat_id,
                &command.auth.chat_type,
                command.auth.chat_title.as_deref(),
            )
            .await?;
        let session = self.db.ensure_active_session(command.auth.chat_id).await?;
        let participants: Vec<crate::domain::models::Participant> =
            self.db.list_participants(command.auth.chat_id).await?;
        let participant_map: HashMap<i64, &crate::domain::models::Participant> = participants
            .iter()
            .map(|participant| (participant.user_id, participant))
            .collect();

        if !participant_map.contains_key(&command.auth.user_id) {
            return Err(AppError::Forbidden(
                "participant must register before adding spends".to_string(),
            ));
        }
        if !participant_map.contains_key(&command.payer_user_id) {
            return Err(AppError::Validation(
                "payer must be a registered participant".to_string(),
            ));
        }
        if command.splits.is_empty() {
            return Err(AppError::Validation("splits are required".to_string()));
        }

        let unique_users: HashSet<i64> = command.splits.iter().map(|split| split.user_id).collect();
        if unique_users.len() != command.splits.len() {
            return Err(AppError::Validation(
                "split users must be unique".to_string(),
            ));
        }

        if unique_users.len() != participants.len() {
            return Err(AppError::Validation(
                "splits must include every registered participant exactly once".to_string(),
            ));
        }

        for split in &command.splits {
            if !participant_map.contains_key(&split.user_id) {
                return Err(AppError::Validation(
                    "split contains an unknown participant".to_string(),
                ));
            }
        }

        let total_cents = parse_money_to_cents(&command.total)?;
        let mode = SpendMode::parse(&command.mode)?;
        let participants_for_snapshot: Vec<SnapshotParticipant> = participants
            .iter()
            .map(SnapshotParticipant::from_domain)
            .collect();

        let (allocations, distribution) = match mode {
            SpendMode::Abs => {
                let mut allocations = Vec::with_capacity(command.splits.len());
                let mut rendered = Vec::with_capacity(command.splits.len());
                let mut total_split_cents = 0i64;

                for split in &command.splits {
                    let share_cents = parse_absolute_share_to_cents(&split.value)?;
                    total_split_cents += share_cents;
                    allocations.push(Allocation {
                        participant_user_id: split.user_id,
                        share_cents,
                    });
                    rendered.push((split.user_id, share_cents));
                }

                if total_split_cents != total_cents {
                    return Err(AppError::Validation(
                        "absolute splits must sum exactly to the total".to_string(),
                    ));
                }

                (allocations, SpendModeDistribution::Abs(rendered))
            }
            SpendMode::Percent => {
                let mut percentages = Vec::with_capacity(command.splits.len());
                let mut percent_sum = Decimal::ZERO;
                for split in &command.splits {
                    let percent = parse_percent(&split.value)?;
                    percent_sum += percent;
                    percentages.push((split.user_id, percent));
                }
                if percent_sum != Decimal::new(100, 0) {
                    return Err(AppError::Validation(
                        "percent splits must sum exactly to 100".to_string(),
                    ));
                }

                let allocations = allocate_percent_splits(total_cents, &percentages)?;
                (allocations, SpendModeDistribution::Percent(percentages))
            }
        };

        self.db
            .create_spend(CreateSpendInput {
                session_id: session.id,
                creator_user_id: command.auth.user_id,
                payer_user_id: command.payer_user_id,
                total_cents,
                mode,
                allocations,
            })
            .await?;

        let snapshot = self
            .build_snapshot(command.auth.chat_id, session.id, command.auth.user_id)
            .await?;
        let creator_name = participant_map
            .get(&command.auth.user_id)
            .map(|participant: &&crate::domain::models::Participant| participant.display_name())
            .unwrap_or_else(|| command.auth.first_name.clone());
        let payer_name = participant_map
            .get(&command.payer_user_id)
            .map(|participant: &&crate::domain::models::Participant| participant.display_name())
            .unwrap_or_else(|| "Unknown".to_string());
        let distribution_text = crate::application::spend_distribution_text(
            mode,
            &distribution,
            &participants_for_snapshot,
        );
        let header = format!(
            "User {creator_name} added new spend {} paid by {payer_name}. Distribution: ",
            format_cents(total_cents)
        );
        let spend_message = format!(
            "{}||{}||",
            escape_markdown_v2(&header),
            escape_markdown_v2(&distribution_text)
        );

        Ok(AddSpendResult {
            snapshot,
            spend_message,
        })
    }
}

fn allocate_percent_splits(
    total_cents: i64,
    percentages: &[(i64, Decimal)],
) -> AppResult<Vec<Allocation>> {
    let total = Decimal::from(total_cents);
    let mut rounded = Vec::with_capacity(percentages.len());
    let mut allocated_cents = 0i64;

    for (user_id, percent) in percentages {
        let raw = total * *percent / Decimal::new(100, 0);
        let floored = raw.floor();
        let floor_cents = floored
            .to_i64()
            .ok_or_else(|| AppError::Validation("allocation is too large".to_string()))?;
        rounded.push((*user_id, floor_cents, raw - floored));
        allocated_cents += floor_cents;
    }

    let mut remaining_cents = total_cents - allocated_cents;
    rounded.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));

    let mut allocations: Vec<Allocation> = rounded
        .iter()
        .map(|(user_id, share_cents, _)| Allocation {
            participant_user_id: *user_id,
            share_cents: *share_cents,
        })
        .collect();

    let mut index = 0usize;
    while remaining_cents > 0 {
        allocations[index].share_cents += 1;
        remaining_cents -= 1;
        index = (index + 1) % allocations.len();
    }

    allocations.sort_by_key(|allocation| allocation.participant_user_id);
    Ok(allocations)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rust_decimal::Decimal;

    use super::allocate_percent_splits;

    #[test]
    fn test_allocate_percent_splits_handles_exact_division() {
        let allocations =
            allocate_percent_splits(1_000, &[(1, Decimal::new(50, 0)), (2, Decimal::new(50, 0))])
                .unwrap();
        assert_eq!(allocations[0].share_cents, 500);
        assert_eq!(allocations[1].share_cents, 500);
    }

    #[test]
    fn test_allocate_percent_splits_uses_largest_remainder() {
        let allocations = allocate_percent_splits(
            100,
            &[
                (30, Decimal::new(3333, 2)),
                (10, Decimal::new(3333, 2)),
                (20, Decimal::new(3334, 2)),
            ],
        )
        .unwrap();
        assert_eq!(
            allocations
                .iter()
                .map(|allocation| (allocation.participant_user_id, allocation.share_cents))
                .collect::<Vec<_>>(),
            vec![(10, 33), (20, 34), (30, 33)]
        );
    }

    #[test]
    fn test_allocate_percent_splits_breaks_ties_by_user_id() {
        let allocations = allocate_percent_splits(
            10,
            &[
                (20, Decimal::new(3333, 2)),
                (10, Decimal::new(3333, 2)),
                (30, Decimal::new(3334, 2)),
            ],
        )
        .unwrap();
        assert_eq!(
            allocations
                .iter()
                .map(|allocation| (allocation.participant_user_id, allocation.share_cents))
                .collect::<Vec<_>>(),
            vec![(10, 3), (20, 3), (30, 4)]
        );
    }
}
