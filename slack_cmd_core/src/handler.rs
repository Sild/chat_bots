use crate::slack_helper::SlackHelper;
use crate::state::HandlerContext;
use anyhow::Result;
use async_trait::async_trait;
use slack_morphism::events::SlackMessageEvent;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait MessageHandler: Send + Sync {
    fn name(&self) -> &str;
    fn short_description(&self) -> &str;
    fn supported_channels(&self) -> &HashSet<String>;
    async fn handle<'a>(
        &self,
        context: HandlerContext<'a>,
        slack_helper: &SlackHelper,
        msg_event: SlackMessageEvent,
        args: Vec<String>,
    ) -> Result<()>;
}

pub const ALL_CHANNELS: &str = "*";

pub type MessageHandlerPtr = Arc<RwLock<Box<dyn MessageHandler>>>;

pub(crate) struct DefaultHelpHandler {
    supported_channels: HashSet<String>,
    channels_help_info: HashMap<String, Vec<String>>,
    all_channels_help_info: Vec<String>,
}

#[async_trait]
impl MessageHandler for DefaultHelpHandler {
    fn name(&self) -> &str {
        "help"
    }

    fn short_description(&self) -> &str {
        "Prints this help message"
    }

    fn supported_channels(&self) -> &HashSet<String> {
        &self.supported_channels
    }

    async fn handle<'a>(
        &self,
        _context: HandlerContext<'a>,
        slack_helper: &SlackHelper,
        msg_event: SlackMessageEvent,
        args: Vec<String>,
    ) -> Result<()> {
        let mut all_info = match self.channels_help_info.get(msg_event.origin.channel.as_ref().unwrap().0.as_str()) {
            Some(v) => v.clone(),
            None => vec![],
        };
        all_info.extend(self.all_channels_help_info.clone());
        all_info.sort();
        let all_info_msg = match all_info.is_empty() {
            true => "".to_string(),
            false => format!("\n• {}", all_info.join("\n• ")),
        };
        let unknown_command_msg = match args.get(0) {
            Some(arg) if arg != self.name() => format!("Unknown command: `{}`\n", arg),
            _ => String::new(),
        };
        let msg = format!(
            "{}Available commands:\n• `{}`: {}{}",
            unknown_command_msg,
            self.name(),
            self.short_description(),
            all_info_msg
        );
        let thread_ts = msg_event.origin.thread_ts.as_ref().unwrap_or(&msg_event.origin.ts);
        slack_helper.send_reply(msg_event.origin.channel.as_ref().unwrap(), thread_ts, &msg).await
    }
}

impl DefaultHelpHandler {
    pub fn new() -> Self {
        Self {
            supported_channels: HashSet::from([ALL_CHANNELS.to_string()]),
            channels_help_info: HashMap::new(),
            all_channels_help_info: vec![],
        }
    }

    pub fn add_handler(&mut self, handler: &Box<dyn MessageHandler>) {
        for channel in handler.supported_channels().iter() {
            match channel.as_ref() {
                ALL_CHANNELS => {
                    self.all_channels_help_info.push(format!("`{}`: {}", handler.name(), handler.short_description()))
                }
                _ => {
                    let channel_help_info = self.channels_help_info.entry(channel.clone()).or_insert_with(|| vec![]);
                    channel_help_info.push(format!("`{}`: {}", handler.name(), handler.short_description()));
                    channel_help_info.sort();
                }
            }
        }
    }
}

pub mod handlers {
    use std::collections::HashSet;
    use std::sync::Arc;
    use async_trait::async_trait;
    use slack_morphism::prelude::SlackMessageEvent;
    use crate::{ALL_CHANNELS, HandlerContext, MessageHandler};
    use crate::slack_helper::SlackHelper;
    use anyhow::Result;
    use tokio::sync::RwLock;

    pub struct JiraHandler {
        host: String,
        token: String,
        supported_channels: HashSet<String>,
    }

    #[async_trait]
    impl MessageHandler for JiraHandler {
        fn name(&self) -> &str {
            "jira"
        }
        fn short_description(&self) -> &str {
            "create jira ticket"
        }
        fn supported_channels(&self) -> &HashSet<String> {
            &self.supported_channels
        }
        async fn handle<'a>(
            &self,
            _context: HandlerContext<'a>,
            _slack_helper: &SlackHelper,
            _msg_event: SlackMessageEvent,
            _: Vec<String>,
        ) -> Result<()> {
            log::info!("Eventually I'll create jira ticket here");
            Ok(())
        }
    }

    impl JiraHandler {
        pub fn new(host: &str, token: &str, supported_channels: HashSet<String>) -> Arc<RwLock<Box<dyn MessageHandler>>> {
            Arc::new(RwLock::new(Box::new(Self {
                host: host.into(),
                token: token.into(),
                supported_channels,
            })))
        }
    }
}
