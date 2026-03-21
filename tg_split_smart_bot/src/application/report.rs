use crate::application::SplitSmartApplication;
use crate::application::types::{AuthenticatedChatRequest, Snapshot};
use crate::domain::money::format_cents;
use crate::domain::telegram_markdown::escape_markdown_v2;
use crate::error::{AppError, AppResult};

impl SplitSmartApplication {
    pub async fn report_for_member(&self, auth: &AuthenticatedChatRequest) -> AppResult<Snapshot> {
        self.db
            .upsert_chat(auth.chat_id, &auth.chat_type, auth.chat_title.as_deref())
            .await?;
        let session = self.db.ensure_active_session(auth.chat_id).await?;
        self.ensure_registered_member(auth.chat_id, auth.user_id)
            .await?;
        self.build_snapshot(auth.chat_id, session.id, auth.user_id)
            .await
    }

    pub async fn render_report(&self, chat_id: i64) -> AppResult<String> {
        let session = self.db.ensure_active_session(chat_id).await?;
        let snapshot = self
            .build_snapshot_without_actor(chat_id, session.id)
            .await?;

        let mut lines = vec![
            "Trip report".to_string(),
            format!("Participants: {}", snapshot.participants.len()),
            format!("Spends: {}", snapshot.spends_count),
            "Balances:".to_string(),
        ];
        for balance in &snapshot.balances {
            let sign = if balance.net_cents > 0 { "+" } else { "" };
            lines.push(format!(
                "{} {}{}",
                balance.display_name,
                sign,
                format_cents(balance.net_cents)
            ));
        }
        lines.push("Transfers:".to_string());
        if snapshot.transfers.is_empty() {
            lines.push("No transfers needed".to_string());
        } else {
            for transfer in &snapshot.transfers {
                lines.push(format!(
                    "{} -> {} {}",
                    transfer.from_name,
                    transfer.to_name,
                    format_cents(transfer.amount_cents)
                ));
            }
        }

        if lines.is_empty() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "report render produced no lines"
            )));
        }

        Ok(escape_markdown_v2(&lines.join("\n")))
    }
}
