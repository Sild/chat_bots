use teloxide::Bot;
use teloxide::prelude::{Message, Requester};

pub struct TGClient {
    bot_impl: Bot,
}

impl TGClient {
    pub fn new() -> Self {
        Self {
            bot_impl: Bot::from_env(),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        log::info!("TGClient started.");
        let bot_cloned = self.bot_impl.clone();
        teloxide::repl(bot_cloned, |bot: Bot, msg: Message| async move {
            log::info!("Received a message: {:?}", msg);
            bot.send_dice(msg.chat.id).await?;
            Ok(())
        }).await;
        log::info!("TGClient finished.");
        Ok(())
    }
}