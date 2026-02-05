use std::sync::Arc;

use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::dptree;
use teloxide::payloads::SendMessageSetters;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{\n+    ChatKind, ChatMemberStatus, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode,\n+    PublicChatKind, WebAppInfo,\n+};
use teloxide::utils::command::BotCommands;
use tracing::info;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::infra::db;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
enum Command {
    #[command(description = "Open the SplitSmart mini app")]
    Start,
    #[command(description = "Show current report")]
    Report,
    #[command(description = "Reset session (admin only)")]
    Reset,
}

pub async fn run(bot: Bot, state: Arc<AppState>) -> anyhow::Result<()> {
    let message_handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command);

    let my_chat_member_handler = Update::filter_my_chat_member()
        .endpoint(handle_my_chat_member);

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(my_chat_member_handler);

    info!("Starting bot long polling");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_command(bot: Bot, state: Arc<AppState>, msg: Message, cmd: Command) -> Result<(), AppError> {
    match cmd {
        Command::Start => {
            ensure_chat_from_message(&state, &msg).await?;
            send_open_button(&bot, &state, msg.chat.id).await?;
        }
        Command::Report => {
            ensure_chat_from_message(&state, &msg).await?;
            let session = db::ensure_active_session(&state.db, msg.chat.id.0).await?;
            let report = crate::application::render_report(&state.db, msg.chat.id.0, &session).await?;
            bot.send_message(msg.chat.id, report)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
        Command::Reset => {
            ensure_chat_from_message(&state, &msg).await?;
            let user_id = msg.from().map(|u| u.id.0).unwrap_or_default();
            ensure_admin(&bot, msg.chat.id.0, user_id).await?;
            let session = db::ensure_active_session(&state.db, msg.chat.id.0).await?;
            let report = crate::application::render_report(&state.db, msg.chat.id.0, &session).await?;
            bot.send_message(msg.chat.id, report)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
            db::end_session(&state.db, session.id).await?;
            db::ensure_active_session(&state.db, msg.chat.id.0).await?;
            bot.send_message(msg.chat.id, "Session reset. A new session has started.")
                .await?;
        }
    }
    Ok(())
}

async fn handle_my_chat_member(bot: Bot, state: Arc<AppState>, upd: ChatMemberUpdated) -> Result<(), AppError> {
    let chat = &upd.chat;
    let was_out = matches!(
        upd.old_chat_member.status(),
        ChatMemberStatus::Left | ChatMemberStatus::Banned
    );
    let is_in = matches!(
        upd.new_chat_member.status(),
        ChatMemberStatus::Member | ChatMemberStatus::Administrator
    );
    let is_bot = upd.new_chat_member.user.is_bot;

    if was_out && is_in && is_bot {
        let chat_id = chat.id;
        let (chat_type, title) = chat_type_and_title(chat);
        db::ensure_chat(&state.db, chat_id.0, chat_type, title.as_deref()).await?;
        send_open_button(&bot, &state, chat_id).await?;
    }
    Ok(())
}

async fn send_open_button(bot: &Bot, state: &AppState, chat_id: ChatId) -> AppResult<()> {
    let webapp_url = format!("{}?chat_id={}", state.config.webapp_url, chat_id.0);
    let browser_url = format!("{}?chat_id={}", state.config.webapp_url, chat_id.0);
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::web_app(
            "Open SplitSmart",
            WebAppInfo { url: webapp_url.clone() },
        )],
        vec![InlineKeyboardButton::url(
            "Open in browser (no auth)",
            browser_url
                .parse()
                .map_err(|_| AppError::Validation("invalid webapp url".to_string()))?,
        )],
    ]);
    bot.send_message(chat_id, "Open SplitSmart 👇")
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

async fn ensure_chat_from_message(state: &AppState, msg: &Message) -> AppResult<()> {
    let (chat_type, title) = chat_type_and_title(&msg.chat);
    db::ensure_chat(&state.db, msg.chat.id.0, chat_type, title.as_deref()).await?;
    Ok(())
}

fn chat_type_and_title(chat: &Chat) -> (&'static str, Option<String>) {
    match &chat.kind {
        ChatKind::Private(_) => ("private", None),
        ChatKind::Public(public) => {
            let chat_type = match &public.kind {
                PublicChatKind::Group(_) => "group",
                PublicChatKind::Supergroup(_) => "supergroup",
                PublicChatKind::Channel(_) => "channel",
            };
            (chat_type, Some(public.title.clone()))
        }
    }
}

async fn ensure_admin(bot: &Bot, chat_id: i64, user_id: i64) -> AppResult<()> {
    if chat_id > 0 {
        return Ok(());
    }
    let member = bot.get_chat_member(chat_id, user_id).await?;
    let status = member.status();
    let allowed = matches!(
        status,
        ChatMemberStatus::Administrator | ChatMemberStatus::Creator
    );
    if !allowed {
        return Err(AppError::Auth("admin permissions required".to_string()));
    }
    Ok(())
}
