use crate::handler::Handler;
use crate::slack_helper::SlackHelper;
use slack_morphism::{SlackApiToken, SlackUser};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) struct State {
    pub(crate) oauth_token: SlackApiToken,
    pub(crate) socket_token: SlackApiToken,
    pub(crate) handlers: Vec<Arc<RwLock<dyn Handler>>>,
    pub(crate) bot_info: SlackUser,
    pub(crate) slack_helper: Arc<RwLock<SlackHelper>>,
}

unsafe impl Send for State {}
unsafe impl Sync for State {}

impl State {
    pub(crate) async fn new(oauth_token: &str, socket_token: &str) -> anyhow::Result<Self> {
        let slack_helper = SlackHelper::new(oauth_token)?;
        let bot_info = slack_helper.get_bot_info().await?;
        Ok(Self {
            oauth_token: SlackApiToken::new(oauth_token.into()),
            socket_token: SlackApiToken::new(socket_token.into()),
            handlers: vec![],
            bot_info,
            slack_helper: Arc::new(RwLock::new(slack_helper)),
        })
    }

    pub(crate) fn add_handlers(&mut self, handlers: Vec<Arc<RwLock<dyn Handler>>>) {
        self.handlers.extend(handlers);
    }
}
