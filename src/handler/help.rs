pub fn handle(_msg: &telebot::objects::Message) -> String {
    return String::from(
        format!("Usage:
/room help
/person help
/person_room help
")
    )
}