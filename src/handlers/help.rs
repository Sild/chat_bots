pub fn handle(_msg: &telebot::objects::Message) -> String {
    return String::from(
        format!("Usage:
/person help
/room help
")
    )
}