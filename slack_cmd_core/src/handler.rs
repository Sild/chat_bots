use std::collections::HashMap;
use anyhow::Result;
use slack_morphism::events::SlackMessageEvent;
use crate::slack_helper::SlackHelper;
use async_trait::async_trait;

#[async_trait]
pub trait MessageHandler {
    fn name(&self) -> String;
    fn short_description(&self) -> String;
    fn supported_channels(&self) -> Vec<String>;
    async fn handle(&self, slack_helper: &SlackHelper, msg_event: SlackMessageEvent, args: Vec<String>) -> Result<()>;
}

pub(crate) struct DefaultHelpHandler {
    name: String,
    short_description: String,
    supported_channels: Vec<String>,
    channels_help_info: HashMap<String, Vec<String>>,
    all_channels_help_info: Vec<String>,
}

impl DefaultHelpHandler {
    pub fn new() -> Self {
        DefaultHelpHandler {
            name: "help".to_string(),
            short_description: "Prints this help message".to_string(),
            supported_channels: vec![String::from("*")],
            channels_help_info: HashMap::new(),
            all_channels_help_info: vec![],
        }
    }

    pub fn add_handler(&mut self, handler: &Box<dyn MessageHandler>) {
        for channel in handler.supported_channels().iter() {
            match channel.as_ref() {
                "*" => self.all_channels_help_info.push(format!("`{}`: {}", handler.name(), handler.short_description())),
                _ => {
                    let channel_help_info = self.channels_help_info.entry(channel.clone()).or_insert_with(|| vec![]);
                    channel_help_info.push(format!("`{}`: {}", handler.name(), handler.short_description()));
                    channel_help_info.sort();
                }
            }

        }
    }
}

#[async_trait]
impl MessageHandler for DefaultHelpHandler {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn short_description(&self) -> String {
        self.short_description.clone()
    }
    fn supported_channels(&self) -> Vec<String> {
        self.supported_channels.clone()
    }
    async fn handle(&self, slack_helper: &SlackHelper, msg_event: SlackMessageEvent, _: Vec<String>) -> Result<()> {
        let mut all_info = match self.channels_help_info.get(msg_event.origin.channel.as_ref().unwrap().0.as_str()) {
            Some(v) => v.clone(),
            None => vec![],
        };
        all_info.extend(self.all_channels_help_info.clone());
        all_info.sort();
        let msg = format!("Available commands:\n• `{}`: {}\n {}", self.name, self.short_description, all_info.join("\n• "));
        let thread_ts = msg_event.origin.thread_ts.as_ref().unwrap_or(&msg_event.origin.ts);
        slack_helper.send_reply(msg_event.origin.channel.as_ref().unwrap(), thread_ts, &msg).await
    }
}
