use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
    pub my_chat_member: Option<ChatMemberUpdated>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub chat: Chat,
    pub new_chat_members: Option<Vec<User>>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String, // "group" | "supergroup" | "private" ...
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChatMemberUpdated {
    pub chat: Chat,
    pub from: User,
    pub old_chat_member: ChatMember,
    pub new_chat_member: ChatMember,
}

#[derive(Debug, Deserialize)]
pub struct ChatMember {
    pub user: User,
    pub status: String, // "left", "kicked", "member", "administrator"...
}

/// ---- sendMessage payload ----

#[derive(Debug, Serialize)]
struct SendMessageReq<'a> {
    chat_id: i64,
    text: &'a str,
    reply_markup: ReplyMarkup<'a>,
}

#[derive(Debug, Serialize)]
struct ReplyMarkup<'a> {
    inline_keyboard: Vec<Vec<InlineKeyboardButton<'a>>>,
}

#[derive(Debug, Serialize)]
struct InlineKeyboardButton<'a> {
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    web_app: Option<WebAppInfo<'a>>,
}

#[derive(Debug, Serialize)]
struct WebAppInfo<'a> {
    url: &'a str,
}

/// Главная функция: если бота добавили в группу — шлём сообщение с кнопкой Mini App.
pub async fn on_update_send_miniapp_button(
    http: &reqwest::Client,
    bot_token: &str,
    miniapp_url: &str,
    update: &Update,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1) Надёжный сигнал: my_chat_member (когда меняется статус именно *бота* в чате)
    if let Some(mcm) = &update.my_chat_member {
        let chat = &mcm.chat;

        // интересуют только группы
        if chat.chat_type == "group" || chat.chat_type == "supergroup" {
            let was_out = matches!(mcm.old_chat_member.status.as_str(), "left" | "kicked");
            let is_in = matches!(
                mcm.new_chat_member.status.as_str(),
                "member" | "administrator"
            );

            if was_out && is_in && mcm.new_chat_member.user.is_bot {
                send_miniapp_entry_message(http, bot_token, chat.id, miniapp_url).await?;
            }
        }
        return Ok(());
    }

    // 2) Запасной вариант: message.new_chat_members (когда добавляют участников)
    if let Some(msg) = &update.message {
        let chat = &msg.chat;
        if chat.chat_type == "group" || chat.chat_type == "supergroup" {
            if let Some(members) = &msg.new_chat_members {
                // если среди добавленных есть бот — значит добавили нас
                let bot_added = members.iter().any(|u| u.is_bot);
                if bot_added {
                    send_miniapp_entry_message(http, bot_token, chat.id, miniapp_url).await?;
                }
            }
        }
    }

    Ok(())
}

pub(super) async fn send_miniapp_entry_message(
    http: &reqwest::Client,
    bot_token: &str,
    chat_id: i64,
    miniapp_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api = format!("https://api.telegram.org/bot{bot_token}/sendMessage");

    let payload = SendMessageReq {
        chat_id,
        text: "Чтобы разделить счёт — откройте SplitSmart 👇",
        reply_markup: ReplyMarkup {
            inline_keyboard: vec![vec![InlineKeyboardButton {
                text: "SplitSmart",
                web_app: Some(WebAppInfo { url: miniapp_url }),
            }]],
        },
    };

    let resp = http.post(api).json(&payload).send().await?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(format!("Telegram sendMessage failed: {status} {body}").into());
    }
    Ok(())
}
