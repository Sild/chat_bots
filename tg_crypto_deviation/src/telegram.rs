use crate::db::ArcDB;
use frankenstein::AsyncTelegramApi;
use frankenstein::GetUpdatesParams;
use frankenstein::Message;
use frankenstein::ReplyParameters;
use frankenstein::SendMessageParams;
use frankenstein::{AsyncApi, UpdateContent};

pub struct TGClient {
    api: AsyncApi,
    db: ArcDB,
}

impl TGClient {
    pub fn new(token: &str, db: ArcDB) -> Self {
        Self { api: AsyncApi::new(token), db }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        log::info!("TGClient started.");
        let param_builder = GetUpdatesParams::builder();
        let mut update_params = param_builder.clone().build();
        loop {
            let result = self.api.get_updates(&update_params).await;
            if let Err(err) = &result {
                log::warn!("Error while getting updates: {err:?}");
                continue;
            }

            for update in result.unwrap().result {
                if let UpdateContent::Message(message) = update.content {
                    let api_clone = self.api.clone();

                    tokio::spawn(async move {
                        process_message(message, api_clone).await;
                    });
                }
                update_params = param_builder.clone().offset(update.update_id + 1).build();
            }
        }
        log::info!("TGClient finished.");
    }
}

async fn process_message(message: Message, api: AsyncApi) {
    let reply_parameters = ReplyParameters::builder().message_id(message.message_id).build();

    let send_message_params = SendMessageParams::builder().chat_id(message.chat.id).text("hello").reply_parameters(reply_parameters).build();

    if let Err(err) = api.send_message(&send_message_params).await {
        println!("Failed to send message: {err:?}");
    }
}
