use crate::application::SplitSmartApplication;
use crate::application::types::{AuthenticatedChatRequest, BootstrapResult};

impl SplitSmartApplication {
    pub async fn bootstrap(
        &self,
        auth: &AuthenticatedChatRequest,
    ) -> crate::error::AppResult<BootstrapResult> {
        self.db
            .upsert_chat(auth.chat_id, &auth.chat_type, auth.chat_title.as_deref())
            .await?;
        let session = self.db.ensure_active_session(auth.chat_id).await?;
        let registration = self
            .db
            .register_participant(
                auth.chat_id,
                auth.user_id,
                auth.username.as_deref(),
                &auth.first_name,
                auth.last_name.as_deref(),
            )
            .await?;
        let snapshot = self
            .build_snapshot(auth.chat_id, session.id, registration.participant.user_id)
            .await?;
        let registration_message = registration.is_new.then(|| {
            format!(
                "User {} registered",
                registration.participant.display_name()
            )
        });

        Ok(BootstrapResult {
            snapshot,
            registration_message,
        })
    }
}
