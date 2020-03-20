use std::env;

use futures::{Future, stream::Stream};

use telebot::Bot;
use telebot::functions::*;

use aalto_tg_bot::handlers;

fn main() {
    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_TOKEN_AALTO").unwrap()).update_interval(200);

    let help = bot.new_cmd("/help")
        .and_then(|(bot, msg)| {
            bot.message(msg.chat.id, handlers::help::handle(&msg)).send()
        })
        .for_each(|_| Ok(()));

    // Register a reply command which answers a message
    let person = bot.new_cmd("/person")
        .and_then(|(bot, msg)| {
            bot.message(msg.chat.id, handlers::person::handle(&msg)).send()
        })
        .for_each(|_| Ok(()));

    let handlers = help.join(person);

    bot.run_with(handlers);
}