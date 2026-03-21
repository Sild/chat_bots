use std::collections::HashMap;

use crate::domain::models::{Allocation, Balance, SpendLedgerEntry, Transfer};

pub fn compute_balances(participant_user_ids: &[i64], spends: &[SpendLedgerEntry]) -> Vec<Balance> {
    let mut balance_map: HashMap<i64, i64> = participant_user_ids
        .iter()
        .copied()
        .map(|user_id| (user_id, 0))
        .collect();

    for spend in spends {
        *balance_map.entry(spend.payer_user_id).or_insert(0) += spend.total_cents;
        for Allocation {
            participant_user_id,
            share_cents,
        } in &spend.allocations
        {
            *balance_map.entry(*participant_user_id).or_insert(0) -= *share_cents;
        }
    }

    let mut balances: Vec<Balance> = balance_map
        .into_iter()
        .map(|(user_id, net_cents)| Balance { user_id, net_cents })
        .collect();
    balances.sort_by_key(|balance| balance.user_id);
    balances
}

pub fn compute_transfers(balances: &[Balance]) -> Vec<Transfer> {
    let mut creditors: Vec<(i64, i64)> = balances
        .iter()
        .filter(|balance| balance.net_cents > 0)
        .map(|balance| (balance.user_id, balance.net_cents))
        .collect();
    let mut debtors: Vec<(i64, i64)> = balances
        .iter()
        .filter(|balance| balance.net_cents < 0)
        .map(|balance| (balance.user_id, balance.net_cents))
        .collect();

    creditors.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    debtors.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));

    let mut transfers = Vec::new();
    let mut creditor_index = 0usize;
    let mut debtor_index = 0usize;

    while creditor_index < creditors.len() && debtor_index < debtors.len() {
        let (creditor_id, creditor_amount) = creditors[creditor_index];
        let (debtor_id, debtor_amount) = debtors[debtor_index];
        let transfer_amount = creditor_amount.min(debtor_amount.abs());

        if transfer_amount > 0 {
            transfers.push(Transfer {
                from_user_id: debtor_id,
                to_user_id: creditor_id,
                amount_cents: transfer_amount,
            });
        }

        creditors[creditor_index].1 -= transfer_amount;
        debtors[debtor_index].1 += transfer_amount;

        if creditors[creditor_index].1 == 0 {
            creditor_index += 1;
        }
        if debtors[debtor_index].1 == 0 {
            debtor_index += 1;
        }
    }

    transfers
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{compute_balances, compute_transfers};
    use crate::domain::models::{Allocation, SpendLedgerEntry};

    #[test]
    fn test_compute_balances_handles_empty_ledger() {
        let balances = compute_balances(&[1, 2], &[]);
        assert_eq!(
            balances
                .iter()
                .map(|entry| entry.net_cents)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );
    }

    #[test]
    fn test_compute_transfers_matches_multiple_debtors_to_one_creditor() {
        let balances = compute_balances(
            &[1, 2, 3],
            &[SpendLedgerEntry {
                payer_user_id: 1,
                total_cents: 1_240,
                allocations: vec![
                    Allocation {
                        participant_user_id: 1,
                        share_cents: 0,
                    },
                    Allocation {
                        participant_user_id: 2,
                        share_cents: 510,
                    },
                    Allocation {
                        participant_user_id: 3,
                        share_cents: 730,
                    },
                ],
            }],
        );
        let transfers = compute_transfers(&balances);
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers[0].from_user_id, 3);
        assert_eq!(transfers[0].to_user_id, 1);
        assert_eq!(transfers[0].amount_cents, 730);
        assert_eq!(transfers[1].from_user_id, 2);
        assert_eq!(transfers[1].to_user_id, 1);
        assert_eq!(transfers[1].amount_cents, 510);
    }

    #[test]
    fn test_compute_transfers_handles_multiple_creditors_and_debtors() {
        let transfers = compute_transfers(&[
            crate::domain::models::Balance {
                user_id: 1,
                net_cents: 500,
            },
            crate::domain::models::Balance {
                user_id: 2,
                net_cents: 300,
            },
            crate::domain::models::Balance {
                user_id: 3,
                net_cents: -200,
            },
            crate::domain::models::Balance {
                user_id: 4,
                net_cents: -600,
            },
        ]);

        assert_eq!(transfers.len(), 3);
        assert_eq!(transfers[0].from_user_id, 4);
        assert_eq!(transfers[0].to_user_id, 1);
        assert_eq!(transfers[0].amount_cents, 500);
        assert_eq!(transfers[1].from_user_id, 4);
        assert_eq!(transfers[1].to_user_id, 2);
        assert_eq!(transfers[1].amount_cents, 100);
        assert_eq!(transfers[2].from_user_id, 3);
        assert_eq!(transfers[2].to_user_id, 2);
        assert_eq!(transfers[2].amount_cents, 200);
    }
}
