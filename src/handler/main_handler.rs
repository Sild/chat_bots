use crate::bot_wrapper;
use std::panic::resume_unwind;

pub fn handle(from: &bot_wrapper::From, msg: &bot_wrapper::Message) -> String {
    let mode = *(msg.data.split(" ").collect::<Vec<&str>>().get(0).unwrap_or(&""));

    println!("new message received: data='{}', mode='{}', first_name='{}', last_name='{}', username='{}'"
        , msg.data, mode, from.first_name, from.last_name, from.username);
    let response = match mode {
        "/help" => crate::handler::help::handle(),
        "/room" => crate::handler::room::handle(msg),
        "/person" => crate::handler::person::handle(from, msg),
        "/person_room" => crate::handler::person_room::handle(from, msg),
        _ => format!("Неизвестный режим: '{}'", mode),
    };
    println!("response=\"\"\"{}\"\"\"", response);
    match response.is_empty() {
        true => String::from("Unexpected error"),
        false => response,
    }
}