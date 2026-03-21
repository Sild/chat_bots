use crate::application::SplitSmartApplication;
use crate::application::types::{AuthenticatedChatRequest, Snapshot};
use crate::error::AppResult;

impl SplitSmartApplication {
    pub async fn reset_for_member(&self, auth: &AuthenticatedChatRequest) -> AppResult<Snapshot> {
        self.db
            .upsert_chat(auth.chat_id, &auth.chat_type, auth.chat_title.as_deref())
            .await?;
        self.ensure_registered_member(auth.chat_id, auth.user_id)
            .await?;
        let session = self.db.reset_session(auth.chat_id).await?;
        self.build_snapshot(auth.chat_id, session.id, auth.user_id)
            .await
    }

    pub async fn reset_session_for_bot(&self, chat_id: i64) -> AppResult<()> {
        self.db.reset_session(chat_id).await?;
        Ok(())
    }
}
