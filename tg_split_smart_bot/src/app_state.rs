use crate::application::SplitSmartApplication;
use crate::infra::telegram::TelegramGateway;

#[derive(Clone)]
pub struct AppState {
    pub application: SplitSmartApplication,
    pub telegram: TelegramGateway,
}

impl AppState {
    pub fn new(application: SplitSmartApplication, telegram: TelegramGateway) -> Self {
        Self {
            application,
            telegram,
        }
    }
}
