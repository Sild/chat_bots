use teloxide::payloads::{PinChatMessageSetters, SendMessageSetters};
use teloxide::prelude::Requester;
use teloxide::types::{
    ChatId, ChatMemberStatus, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode, UserId,
    WebAppInfo,
};
use tracing::warn;

use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct TelegramGateway {
    bot: teloxide::Bot,
    app_base_url: String,
}

impl TelegramGateway {
    pub fn new(bot: teloxide::Bot, app_base_url: String) -> Self {
        Self { bot, app_base_url }
    }

    pub async fn send_open_message(&self, chat_id: i64) -> AppResult<()> {
        let url = self.open_app_url(chat_id);
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::web_app(
                "Open SplitSmart",
                WebAppInfo {
                    url: url::Url::parse(&url)
                        .map_err(|_| AppError::Validation("invalid app url".to_string()))?,
                },
            )],
            vec![InlineKeyboardButton::url(
                "Open in browser (no auth)",
                url.parse()
                    .map_err(|_| AppError::Validation("invalid app url".to_string()))?,
            )],
        ]);

        let message = self
            .bot
            .send_message(ChatId(chat_id), "Open SplitSmart 👇")
            .reply_markup(keyboard)
            .await?;

        if let Err(error) = self
            .bot
            .pin_chat_message(ChatId(chat_id), message.id)
            .disable_notification(true)
            .await
        {
            warn!(chat_id, ?error, "failed to pin open message");
        }

        Ok(())
    }

    pub async fn send_registration_message(&self, chat_id: i64, text: &str) -> AppResult<()> {
        self.bot.send_message(ChatId(chat_id), text).await?;
        Ok(())
    }

    pub async fn send_spend_message(&self, chat_id: i64, text: &str) -> AppResult<()> {
        self.send_markdown_message(chat_id, text).await
    }

    pub async fn send_report_message(&self, chat_id: i64, text: &str) -> AppResult<()> {
        self.send_markdown_message(chat_id, text).await
    }

    pub async fn ensure_admin(&self, chat_id: i64, user_id: i64) -> AppResult<()> {
        if chat_id > 0 {
            return Ok(());
        }

        let member = self
            .bot
            .get_chat_member(ChatId(chat_id), UserId(user_id as u64))
            .await?;
        let is_admin = matches!(
            member.status(),
            ChatMemberStatus::Administrator | ChatMemberStatus::Owner
        );
        if !is_admin {
            return Err(AppError::Forbidden(
                "admin permissions required for reset".to_string(),
            ));
        }

        Ok(())
    }

    pub fn open_app_url(&self, chat_id: i64) -> String {
        format!("{}?chat_id={chat_id}", self.app_base_url)
    }

    pub fn bot(&self) -> &teloxide::Bot {
        &self.bot
    }

    async fn send_markdown_message(&self, chat_id: i64, text: &str) -> AppResult<()> {
        self.bot
            .send_message(ChatId(chat_id), text.to_string())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}
