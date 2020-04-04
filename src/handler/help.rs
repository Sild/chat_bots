pub fn handle(msg: &telebot::objects::Message) -> String {
    println!("\nnew request: '/help {}'", msg.text.as_ref().unwrap());

    return String::from(
        format!("Usage:
/room help
/person help
/person_room help
")
    )
}