use crate::util;
use crate::db::model::{Person, PersonRole, Room, PersonRoom};
use dict::DictIface;
use crate::bot_wrapper;


fn help(prefix: &str) -> String {
    return format!("{}
Использование:
/person_room help
/person_room link tg_login='' room_num=''
/person_room unlink tg_login='' room_num=''
    ", prefix);
}

fn link_unlink_impl(args: &Vec<&str>, remove: bool) -> String {
    let kwargs = util::parse_kwargs(args.join(" ").as_str());

    let tg_login: String = kwargs.get("tg_login").unwrap_or(&String::new()).to_string();
    let room_num: u32 = kwargs.get("room_num").unwrap_or(&String::new()).to_string().parse().unwrap_or(0);

    if tg_login.is_empty() || room_num == 0 {
        return String::from("Ошибка: некорректные параметры");
    }
    let person = Person::select_by_tg_logins(&vec!(tg_login.clone()));
    let rooms = Room::select_by_room_nums(&vec!(room_num));
    if person.len() < 1 {
        return format!("Не нашли пользователя с tg_login='{}'", tg_login);
    }
    let person = person.get(0).unwrap();
    if rooms.len() < 1 {
        return format!("Не нашли квартиру с номером='{}'", room_num);
    }
    let mut person_rooms = PersonRoom::select_by_room_ids(&vec!(rooms[0].id));
    for pr in &mut person_rooms {
        if pr.person_id == person.id {
            if remove {
                pr.delete();
                return String::from("Удалено.");
            } else {
                return format!("Такая связь уже существует");
            }
        }
    }
    if remove {
        return String::from("Связь не найдена");
    }
    let mut person_room = PersonRoom {
        id: 0,
        person_id: person.id,
        room_id: rooms[0].id
    };
    person_room.save();
    return format!("Сохранено: {} (tg_login: '{}', квартира: {})", person_room.to_string(), person.tg_login, rooms[0].num);
}

pub fn handle(from: &bot_wrapper::From, msg: &bot_wrapper::Message) -> String {
    let arguments: Vec<&str> = msg.data.split(" ").collect();
    let command = arguments.get(1).unwrap_or(&"/help");

    let who = match Person::select_by_tg_logins(&vec!(from.username.clone())).get(0) {
        Some(x) => x.clone(),
        _ => Person::new(),
    };
    if who.role != PersonRole::Admin {
        return format!("Ошибка: пользователь tg_login='{}' не найден или не является администратором.", from.username);
    }

    return match *command {
        "help" => help(""),
        "link" => link_unlink_impl(arguments.as_ref(), false),
        "unlink" => link_unlink_impl(arguments.as_ref(), true),
        _ => help("Неизвесная команда")
    };
}