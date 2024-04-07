use crate::handler::{DefaultHelpHandler, MessageHandler};
use crate::slack_helper::SlackHelper;
use slack_morphism::{SlackApiToken, SlackBotInfo};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) struct State {
    pub(crate) handlers: Vec<Arc<RwLock<dyn MessageHandler>>>,
    pub(crate) bot_marker: String,
    pub(crate) bot_info: SlackBotInfo,
    pub(crate) default_helper: Option<DefaultHelpHandler>,
    pub(crate) slack_helper: Arc<RwLock<SlackHelper>>,
}

unsafe impl Send for State {}
unsafe impl Sync for State {}

impl State {
    pub async fn new(oauth_token: &str) -> anyhow::Result<Self> {
        let slack_helper = SlackHelper::new(oauth_token)?;
        let bot_info = slack_helper.get_bot_info().await?;
        Ok(Self {
            handlers: vec![],
            bot_marker: format!("<@{}> ", bot_info.user_id),
            bot_info,
            default_helper: None,
            slack_helper: Arc::new(RwLock::new(slack_helper)),
        })
    }

    pub async fn new_with_default_helper(oauth_token: &str) -> anyhow::Result<Self> {
        let mut state = Self::new(oauth_token).await?;
        let helper = crate::handler::DefaultHelpHandler::new();
        state.default_helper = Some(helper);
        Ok(state)
    }

    pub(crate) fn add_handlers(&mut self, handlers: Vec<Arc<RwLock<dyn MessageHandler>>>) {
        self.handlers.extend(handlers);
    }
}
