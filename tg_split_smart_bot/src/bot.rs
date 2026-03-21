mod commands;
mod handler;

use std::sync::Arc;

use crate::app_state::AppState;

#[derive(Clone)]
pub struct SplitSmartBot {
    state: Arc<AppState>,
}

impl SplitSmartBot {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        handler::run(self.state).await
    }
}
