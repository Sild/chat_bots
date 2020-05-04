use crate::bot_wrapper;

pub fn handle(msg: &bot_wrapper::Message) -> String {
    println!("\nnew request: '/help {}'", msg.data);

    return String::from(
        format!("Usage:
/room help
/person help
/person_room help
")
    )
}