use crate::util;
use crate::db::model::{Person, PersonRole, PersonRoom, Room};
use dict::DictIface;
use std::borrow::Borrow;


fn help(prefix: &str) -> String {
    return format!("{}
Использование:
/person help
/person add tg_login='' phone='' email='' name=''
/person update id='' tg_login='' phone='' email='' name=''
/person delete id1 id2 id3
/person info id1 id2 id3
/person admins

изменения, почта и номер телефона доступны только администраторам
    ", prefix);
}

fn add_or_update(args: &Vec<&str>, who: &Person) -> String {
    let cmd = args.join(" ");
    let kwargs = util::parse_kwargs(cmd.as_str());

    let id: u32 = kwargs.get("id").unwrap_or(&String::new()).parse().unwrap_or(0);
    let tg_login: String = kwargs.get("tg_login").unwrap_or(&String::new()).to_string();
    let phone: String = kwargs.get("phone").unwrap_or(&String::new()).to_string();
    let email: String = kwargs.get("email").unwrap_or(&String::new()).to_string();
    let fio: String = kwargs.get("name").unwrap_or(&String::new()).to_string();
    let role_str: String = kwargs.get("role").unwrap_or(&String::new()).to_string();

    let role = match role_str.as_str() {
        "" => PersonRole::User,
        _ => PersonRole::from_str(role_str.as_str()),
    };

    let mut person: Person;
    if id != 0 {
        let mut stored_persons = Person::select_by_ids(vec![id].as_ref());
        if stored_persons.len() < 1 {
            return format!("Нет пользователя с id='{}'", id);
        } else {
            person = stored_persons.get(0).unwrap().clone();
        }
        if !tg_login.is_empty() {
            person.tg_login = tg_login
        }
        if !phone.is_empty() {
            person.phone = phone
        }
        if !email.is_empty() {
            person.email = email
        }
        if !fio.is_empty() {
            person.fio = fio
        }
        if !role_str.is_empty() {
            person.role = role
        }
    } else {
        person = Person {
            id, tg_login, phone, email, fio, role
        };
    }
    person.save();
    if person.id == 0 {
        return format!("Ошибка: не удалось сохранить пользователя: {}", person.to_string(&who.role));
    }
    return format!("Сохранено: {}", person.to_string(&who.role));
}

fn delete(args: &Vec<&str>) -> String {
    let mut response = String::new();
    let mut person_ids = Vec::<u32>::new();

    for i in 1..args.len() {
        match args[i].parse::<u32>() {
            Ok(t) => person_ids.push(t),
            _ => response.push_str(format!("Ошибка: странный id пользователя: '{}'\n", args[i]).as_str()),
        };
    }
    Person::delete_by_ids(&person_ids);
    response.push_str(format!("Удаленные id: {}"
                      , person_ids.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ")).as_str());
    return response;
}

fn info(args: &Vec<&str>, who: &Person) -> String {
    if args.len() < 2 {
        return help("Не достаточно аргументов: введите хотя бы 1 номер квартиры");
    }

    let mut response = String::new();
    let mut person_ids = Vec::<u32>::new();

    for i in 1..args.len() {
        match args[i].parse::<u32>() {
            Ok(t) => person_ids.push(t),
            _ => response.push_str(format!("Ошибка: странный id пользователя: '{}'\n", args[i]).as_str()),
        };
    }
    let persons = Person::select_by_ids(&person_ids);
    let person_rooms = PersonRoom::select_by_person_ids(
        persons.iter().map(|x| x.id).collect::<Vec<u32>>().as_ref()
    );
    let rooms = Room::select_by_room_nums(
        person_rooms.iter().map(|x| x.room_id).collect::<Vec<u32>>().as_ref()

    );
    response.push_str(
        util::format_response_person_info(persons.as_ref(),rooms.as_ref(), person_rooms.as_ref(), &who.role).as_str()
    );
    println!("{}", response);
    return response;
}

fn admins(who: &Person) -> String {
    let mut response = String::new();
    for p in Person::select_admins() {
        response.push_str(format!("{}\n", p.to_string(&who.role)).as_str());
    }
    return response;
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
    let who_tg_login = msg.from.as_ref().unwrap().username.as_ref().unwrap();

    let arguments: Vec<&str> = msg.text.as_ref().unwrap().split(" ").collect();
    let who = Person::select_by_tg_login(who_tg_login);

    if who.is_none() {
        return format!("Ошибка: пользователь с tg_login='{}' не найден.", msg.from.as_ref().unwrap().username.as_ref().unwrap_or(&String::new()));
    }
    let who = who.unwrap();

    if who.role != PersonRole::Admin && !vec!("help", "info", "admins").contains(&arguments[0]) {
    }

    return match arguments[0] {
        "help" => help(""),
        "add" => add_or_update(arguments.as_ref(), &who),
        "update" => add_or_update(arguments.as_ref(), &who),
        "delete" => delete(arguments.as_ref()),
        "info" => info(arguments.as_ref(), &who),
        "admins" => admins(&who),
        _ => help("Unknown command")
    };
}