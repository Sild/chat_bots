use std::sync::Arc;
use teloxide::{dptree};
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Requester, Update};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use tracing::{info, warn};
use crate::app_state::AppState;

pub struct TGBot {
    state: Arc<AppState>,
    inner: teloxide::Bot,
}

impl TGBot {
    pub fn new(state: Arc<AppState>, token: &str) -> anyhow::Result<Self> {
        let bot = teloxide::Bot::new(token);
        tokio::spawn(run_event_handling(bot.clone()));
        Ok(Self {
            state,
            inner: bot,
        })
    }
}

async fn run_event_handling(bot: teloxide::Bot) -> anyhow::Result<()> {
    // Handler for when bot is added to a group via my_chat_member update
    let my_chat_member_handler = Update::filter_my_chat_member().endpoint(move |bot: teloxide::Bot, upd: teloxide::types::ChatMemberUpdated| {
        async move {
            let chat = &upd.chat;
            info!(chat_id = %chat.id, old_status = ?upd.old_chat_member.status(), new_status = ?upd.new_chat_member.status(), is_bot = upd.new_chat_member.user.is_bot, "my_chat_member update received");
            let was_out = matches!(upd.old_chat_member.status(), teloxide::types::ChatMemberStatus::Left | teloxide::types::ChatMemberStatus::Banned);
            let is_in = matches!(upd.new_chat_member.status(), teloxide::types::ChatMemberStatus::Member | teloxide::types::ChatMemberStatus::Administrator);
            let is_bot = upd.new_chat_member.user.is_bot;
            if was_out && is_in && is_bot {
                info!(chat_id = %chat.id, "Detected bot added to chat; sending SplitSmart button");

                let ikb = InlineKeyboardMarkup::new([[InlineKeyboardButton::url("SplitSmart", "https://t.me/split_smart_bot/app?chat_id={chat_id}".parse().unwrap())]]);
                bot
                    .send_message(chat.id, "Чтобы разделить счёт — откройте SplitSmart 👇")
                    .reply_markup(ikb)
                    .await?;
            }
            Ok::<(), teloxide::RequestError>(())
        }
    });

    let handler = dptree::entry()
        .branch(my_chat_member_handler);

    info!("Starting bot polling");
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
