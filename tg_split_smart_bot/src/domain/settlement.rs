use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Allocation {
    pub participant_user_id: i64,
    pub share_cents: i64,
}

#[derive(Debug, Clone)]
pub struct Spend {
    pub payer_user_id: i64,
    pub total_cents: i64,
    pub allocations: Vec<Allocation>,
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub user_id: i64,
    pub net_cents: i64,
}

#[derive(Debug, Clone)]
pub struct Transfer {
    pub from_user_id: i64,
    pub to_user_id: i64,
    pub amount_cents: i64,
}

pub fn compute_balances(spends: &[Spend]) -> HashMap<i64, i64> {
    let mut balances: HashMap<i64, i64> = HashMap::new();
    for spend in spends {
        *balances.entry(spend.payer_user_id).or_insert(0) += spend.total_cents;
        for alloc in &spend.allocations {
            *balances.entry(alloc.participant_user_id).or_insert(0) -= alloc.share_cents;
        }
    }
    balances
}

pub fn compute_transfers(balances: &HashMap<i64, i64>) -> Vec<Transfer> {
    let mut creditors: Vec<(i64, i64)> = balances
        .iter()
        .filter(|(_, v)| **v > 0)
        .map(|(k, v)| (*k, *v))
        .collect();
    let mut debtors: Vec<(i64, i64)> = balances
        .iter()
        .filter(|(_, v)| **v < 0)
        .map(|(k, v)| (*k, *v))
        .collect();

    creditors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    debtors.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

    let mut transfers = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < debtors.len() && j < creditors.len() {
        let (debtor_id, debtor_amt) = debtors[i];
        let (creditor_id, creditor_amt) = creditors[j];
        let owe = debtor_amt.abs();
        let pay = owe.min(creditor_amt);
        if pay > 0 {
            transfers.push(Transfer {
                from_user_id: debtor_id,
                to_user_id: creditor_id,
                amount_cents: pay,
            });
        }
        let new_debtor_amt = debtor_amt + pay;
        let new_creditor_amt = creditor_amt - pay;
        debtors[i].1 = new_debtor_amt;
        creditors[j].1 = new_creditor_amt;
        if debtors[i].1 == 0 {
            i += 1;
        }
        if creditors[j].1 == 0 {
            j += 1;
        }
    }

    transfers
}
