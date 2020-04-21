use std::env;

use futures::{Future, stream::Stream};

use telebot::Bot;
use telebot::functions::*;

use aalto_tg_bot::handler;
use aalto_tg_bot::db::init::init_db;
use aalto_tg_bot::config;

fn format_result(response: &str) -> String {
    return format!("Результат:\n{}", response);
}

fn main() {
    init_db();

    let mut bot = Bot::new(&env::var("TELEGRAM_BOT_TOKEN_AALTO").unwrap()).update_interval(100);

    let help = bot.new_cmd("/help")
        .and_then(|(bot, msg)| {
            bot.message(msg.chat.id, format_result(handler::help::handle(&msg).as_str())).send()
        })
        .for_each(|_| Ok(()));

    // Register a reply command which answers a message
    let person = bot.new_cmd("/person")
        .and_then(|(bot, msg)| {
            bot.message(msg.chat.id, format_result(handler::person::handle(&msg).as_str())).send()
        })
        .for_each(|_| Ok(()));

    let room = bot.new_cmd("/room")
        .and_then(|(bot, msg)| {
            bot.message(msg.chat.id, format_result(handler::room::handle(&msg).as_str())).send()
        })
        .for_each(|_| Ok(()));

    let person_room = bot.new_cmd("/person_room")
        .and_then(|(bot, msg)| {
            bot.message(msg.chat.id, format_result(handler::person_room::handle(&msg).as_str())).send()
        })
        .for_each(|_| Ok(()));

    let dump_db = bot.new_cmd("/dump_db")
        .and_then(|(bot, msg)| {
            if msg.from.unwrap().username.unwrap() == "sildtm" {
                bot.document(msg.chat.id)
                    .file(config::DB_PATH)
                    .send()
            } else {
                bot.document(msg.chat.id)
                    .file(("empty.sqlite", "".as_bytes()))
                    .send()
            }
        })
        .for_each(|_| Ok(()));

    let handlers = help.join(person).join(room).join(person_room).join(dump_db);

    bot.run_with(handlers);
}