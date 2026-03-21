use crate::application::SplitSmartApplication;
use crate::infra::telegram::TelegramGateway;

#[derive(Clone)]
pub struct AppState {
    pub application: SplitSmartApplication,
    pub telegram: TelegramGateway,
    pub telegram_bot_username: String,
}

impl AppState {
    pub fn new(
        application: SplitSmartApplication,
        telegram: TelegramGateway,
        telegram_bot_username: String,
    ) -> Self {
        Self {
            application,
            telegram,
            telegram_bot_username,
        }
    }
}
