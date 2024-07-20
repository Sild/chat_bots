use std::collections::HashMap;
use std::sync::Arc;
use moka::sync::Cache;

pub type ID = u64;
pub type TgID = String;
pub type Symbol = String;


#[derive(Debug, Clone)]
pub struct Alert {
    pub id: ID,
    pub user_id: TgID,
    pub user_alert_id: ID,
    pub symbol: Symbol,
    pub init_value: f64, // price on the moment when alert was created
    pub value: f64,
    pub create_ts: u64,
}

pub type ArcDB = Arc<DB>;
pub struct DB {
    alerts_cache: Cache<Symbol, HashMap<ID, Alert>>
}

impl DB {
    pub fn new() -> Self {
        Self {
            alerts_cache: Cache::new(100),
        }
    }

    pub fn add_alert(&self, alert: Alert) -> anyhow::Result<()> {
        // let alerts = self.alerts_cache.get(&alert.symbol).unwrap_or_else(|| vec![]);
        // let new_alerts = alerts.into_iter().chain(std::iter::once(alert)).collect();
        // self.alerts_cache.insert(alert.symbol, new_alerts);
        Ok(())
    }

    pub fn get_alerts(&self, symbol: &Symbol) -> anyhow::Result<Vec<Alert>> {
        // self.alerts_cache.get(symbol).unwrap_or_else(|| vec![])
        Ok(vec![])
    }

    pub fn delete_alert(&self, alert_id: ID) -> anyhow::Result<()> {
        // let mut alerts = self.alerts_cache.iter().collect::<Vec<_>>();
        // for (symbol, alerts) in alerts.iter_mut() {
        //     let new_alerts = alerts.into_iter().filter(|alert| alert.id != alert_id).collect();
        //     self.alerts_cache.insert(symbol.clone(), new_alerts);
        // }
        Ok(())

    }
}