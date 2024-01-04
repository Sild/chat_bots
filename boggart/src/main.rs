use std::env;
use futures::StreamExt;
use telegram_bot::{Error, Api};
use telegram_bot::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_BOT_TOKEN_AALTO").expect("TELEGRAM_BOT_TOKEN_AALTO not set");
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;
        if let telegram_bot::UpdateKind::Message(message) = update.kind {
            let msg = ::tvoi_sosed::bot_wrapper::Message::from(&message);
            let from = ::tvoi_sosed::bot_wrapper::From::from(&message.from);
            let response = ::tvoi_sosed::handler::main_handler::handle(&from, &msg);
            api.send(message.text_reply(response)).await?;
        }
    }
    Ok(())
}