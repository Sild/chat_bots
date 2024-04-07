use std::cell::RefCell;
use crate::handler::{DefaultHelpHandler, MessageHandler, MessageHandlerPtr};
use crate::slack_helper::SlackHelper;
use anyhow::{anyhow, Result};
use slack_morphism::{SlackApiToken, SlackBotInfo, SlackChannelId};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::ErrorKind::InvalidInput;
use std::io::{Error, ErrorKind};
use std::mem::swap;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::ALL_CHANNELS;

pub(crate) struct Context {
    pub(crate) bot_marker: String,
    pub(crate) bot_info: SlackBotInfo,
    pub(crate) slack_helper: Arc<RwLock<SlackHelper>>,
    // must guarantee maps are always up to date using system handlers
    // don't forget to rebuild handlers index
    // TODO
    pub(crate) channel_map: HashMap<String, SlackChannelId>, // name -> id
    pub(crate) channel_reverse_map: HashMap<SlackChannelId, String>, // id -> name
    pub(crate) default_help_handler: DefaultHelpHandler,
    handlers: Vec<MessageHandlerPtr>,
    // channel_id -> handler_name -> handler
    // if handler support all channels, it'll be added to for all channel_ids
    handlers_index: RefCell<HashMap<SlackChannelId, HashMap<String, MessageHandlerPtr>>>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    pub async fn new(oauth_token: &str) -> Result<Self> {
        let slack_helper = SlackHelper::new(oauth_token)?;
        let bot_info = slack_helper.get_bot_info().await?;
        Ok(Self {
            bot_marker: format!("<@{}>", bot_info.user_id),
            bot_info,
            slack_helper: Arc::new(RwLock::new(slack_helper)),
            channel_map: HashMap::default(), // TODO fill
            channel_reverse_map: HashMap::default(),
            default_help_handler: DefaultHelpHandler::new(),
            handlers: vec![],
            handlers_index: RefCell::new(HashMap::default()),
        })
    }

    pub(crate) async fn add_handlers(&mut self, handlers: Vec<Arc<RwLock<Box<dyn MessageHandler>>>>) -> Result<()> {
        self.handlers = handlers;
        self.rebuild_handlers_index().await
    }

    async fn rebuild_handlers_index(&mut self) -> Result<()> {
        self.handlers_index.borrow_mut().clear();

        // reset default_help_handler because we must rebuild channel<->handlers info
        self.default_help_handler = DefaultHelpHandler::new();
        for handler_ptr in self.handlers.iter() {
            let handler = handler_ptr.read().await;
            if handler.supported_channels().contains(ALL_CHANNELS) {
                // add handler to all available channel_ids
                for (channel_id, channel_name) in self.channel_reverse_map.iter() {
                    self.add_handler_to_index(channel_id, channel_name, handler.name(), handler_ptr.clone())?;
                }
            }
            for channel_name in handler.supported_channels().iter() {
                let channel_id = match self.channel_map.get(channel_name) {
                    Some(id) => id,
                    None => continue,
                };
                self.add_handler_to_index(channel_id, channel_name, handler.name(), handler_ptr.clone())?;
            }
            self.default_help_handler.add_handler(handler.deref());
            log::info!(
                "registered message_handler with name: {}, for channels: {:?}",
                handler.name(),
                handler.supported_channels()
            );
        }
        Ok(())
    }

    fn add_handler_to_index(&self, channel_id: &SlackChannelId, channel_name: &str, handler_name: &str, handler_ptr: MessageHandlerPtr) -> Result<()> {
        let mut handler_index = self.handlers_index.borrow_mut();
        let cur_handlers_index = handler_index.entry(channel_id.clone()).or_insert_with(HashMap::default);
        match self.channel_map.get(handler_name) {
            Some(_) => {
                let err_msg = format!(
                    "got many message_handler name='{}' for channel_name='{}'",
                    handler_name,
                    channel_name
                );
                return Err(anyhow!(Error::new(InvalidInput, err_msg)));
            }
            None => {
                cur_handlers_index.insert(handler_name.to_string(), handler_ptr.clone());
            }
        }
        Ok(())
    }

    pub(crate) fn get_message_handler(
        &self,
        channel_id: &SlackChannelId,
        handler_name: &str,
    ) -> Option<MessageHandlerPtr> {
        Some(self.handlers_index.borrow().get(channel_id)?.get(handler_name)?.clone())
    }
}

pub struct HandlerContext<'a> {
    pub bot_marker: &'a String,
    pub bot_info: &'a SlackBotInfo,
    pub channel_map: &'a HashMap<String, SlackChannelId>, // name -> id
    pub channel_reverse_map: &'a HashMap<SlackChannelId, String>, // id -> name
}

impl<'a> From<&'a Context> for HandlerContext<'a> {
    fn from(value: &'a Context) -> Self {
        Self {
            bot_marker: &value.bot_marker,
            bot_info: &value.bot_info,
            channel_map: &value.channel_map,
            channel_reverse_map: &value.channel_reverse_map,
        }
    }
}
