use std::sync::Arc;

use teloxide::dispatching::HandlerExt;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::dptree;
use teloxide::prelude::{Bot, Dispatcher, Message, Update};
use teloxide::types::{ChatKind, ChatMemberStatus, ChatMemberUpdated, PublicChatKind};
use tracing::warn;

use crate::app_state::AppState;
use crate::bot::commands::Command;
use crate::error::AppError;

pub(super) async fn run(state: Arc<AppState>) -> anyhow::Result<()> {
    let message_handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(handle_command);
    let chat_member_handler = Update::filter_my_chat_member().endpoint(handle_my_chat_member);

    Dispatcher::builder(
        bot_for_dispatcher(&state),
        dptree::entry()
            .branch(message_handler)
            .branch(chat_member_handler),
    )
    .dependencies(dptree::deps![state])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
    Ok(())
}

fn bot_for_dispatcher(state: &AppState) -> teloxide::Bot {
    state.telegram.bot().clone()
}

async fn handle_command(
    _bot: Bot,
    state: Arc<AppState>,
    message: Message,
    command: Command,
) -> Result<(), AppError> {
    let chat_id = message.chat.id.0;
    let (chat_type, title) = chat_metadata(&message.chat);
    state
        .application
        .upsert_chat_metadata(chat_id, chat_type, title.as_deref())
        .await?;

    match command {
        Command::Start => state.telegram.send_open_message(chat_id).await?,
        Command::Report => {
            let report = state.application.render_report(chat_id).await?;
            state.telegram.send_report_message(chat_id, &report).await?;
        }
        Command::Reset => {
            let requester_user_id = message
                .from()
                .map(|user| i64::try_from(user.id.0).unwrap_or_default())
                .ok_or_else(|| {
                    AppError::Forbidden("missing Telegram user on command".to_string())
                })?;
            state
                .telegram
                .ensure_admin(chat_id, requester_user_id)
                .await?;
            let report = state.application.render_report(chat_id).await?;
            state.telegram.send_report_message(chat_id, &report).await?;
            state.application.reset_session_for_bot(chat_id).await?;
        }
    }

    Ok(())
}

async fn handle_my_chat_member(
    _bot: Bot,
    state: Arc<AppState>,
    update: ChatMemberUpdated,
) -> Result<(), AppError> {
    let was_out = matches!(
        update.old_chat_member.status(),
        ChatMemberStatus::Left | ChatMemberStatus::Banned
    );
    let is_in = matches!(
        update.new_chat_member.status(),
        ChatMemberStatus::Member | ChatMemberStatus::Administrator
    );

    if was_out && is_in && update.new_chat_member.user.is_bot {
        let chat_id = update.chat.id.0;
        let (chat_type, title) = chat_metadata(&update.chat);
        state
            .application
            .upsert_chat_metadata(chat_id, chat_type, title.as_deref())
            .await?;
        if let Err(error) = state.telegram.send_open_message(chat_id).await {
            warn!(
                chat_id,
                ?error,
                "failed to send open message after chat member update"
            );
        }
    }

    Ok(())
}

fn chat_metadata(chat: &teloxide::types::Chat) -> (&'static str, Option<String>) {
    let title = chat.title().map(str::to_string);
    match &chat.kind {
        ChatKind::Private(_) => ("private", None),
        ChatKind::Public(public) => match &public.kind {
            PublicChatKind::Group(_) => ("group", title),
            PublicChatKind::Supergroup(_) => ("supergroup", title),
            PublicChatKind::Channel(_) => ("channel", title),
        },
    }
}
