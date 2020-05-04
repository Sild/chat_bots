use std::env;
use bot_wrapper;
use futures::StreamExt;
use telegram_bot::{Error, Api, Message};
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
            let msg = bot_wrapper::Message::from(message);
            if let telegram_bot::MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Answer message with "Hi".
                api.send(message.text_reply(format!(
                    "Hi, {}! You just wrote '{}'",
                    &message.from.first_name, data
                )))
                    .await?;
            }
        }
    }
    Ok(())
}