fn help(prefix: &str) -> String {
    return format!("{}
Использование:
/person help
/person add tg_login='' phone='' email='' fio=''
/person remove id=''
/person info id='' tg_login='' phone=''
/person link_room person_id room_num
/person unlink_room person_id room_num
/person admin

изменения, почта и номер телефона доступны только администраторам
    ", prefix);
}

fn add(args: &Vec<&str>) -> String {
    return String::from("add");
}

fn remove(args: &Vec<&str>) -> String {
//    person_db::add();
    return String::from("remove");
}

fn info(args: &Vec<&str>) -> String {
//    person_db::add();
    return String::from("info");
}

fn link_room(args: &Vec<&str>) -> String {
//    person_db::add();
    return String::from("link_room");
}

fn unlink_room(args: &Vec<&str>) -> String {
//    person_db::add();
    return String::from("unlink_room");
}

pub fn handle(msg: &telebot::objects::Message) -> String {
    let arguments: Vec<&str> = msg.text.as_ref().unwrap().split(" ").collect();

    return match arguments[0] {
        "help" => help(""),
        "add" => add(arguments.as_ref()),
        "remove" => remove(arguments.as_ref()),
        "info" => info(arguments.as_ref()),
        "link_room" => link_room(arguments.as_ref()),
        "unlink_room" => unlink_room(arguments.as_ref()),
        _ => help("Unknown command")
    };
}