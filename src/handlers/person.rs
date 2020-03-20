use crate::person_db;

fn help(prefix: &str) -> String {
    let mut res = String::from(prefix);
    res.push_str("\n");
    res.push_str("
Usage:
/person add tg_login phone email fio
/person remove id
/person search room={} section={} floor={}
/person info id={} tg_login={} phone={}
/person link_room person_id room_num
/person unlink_room person_id room_num

* - admins also receive phone + email
    ");
    return res;
}

fn add() -> String {
    person_db::add();
    return String::from("add");
}

pub fn handle(msg: &telebot::objects::Message) -> String {
    let arguments: Vec<&str> = match msg.text.as_ref() {
        None => std::vec::Vec::new(),
        Some(text) => text.split(" ").collect(),
    };
    if arguments.len() == 0 {
        return help("An arguments required");
    }
    return match arguments[0] {
        "add" => add(),
        _ => help("Unknown command")
    };
}