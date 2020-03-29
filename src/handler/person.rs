use crate::util;
use crate::db::model::{Person, PersonRole};
use dict::DictIface;


fn help(prefix: &str) -> String {
    return format!("{}
Использование:
/person help
/person add tg_login='' phone='' email='' name=''
/person update id='' tg_login='' phone='' email='' name=''
/person remove id1 id2 id3
/person info id1 id2 id3
/person link_room person_id room_num
/person unlink_room person_id room_num
/person admin

изменения, почта и номер телефона доступны только администраторам
    ", prefix);
}

fn add_or_update(args: &Vec<&str>, tg_login_from: &str) -> String {
     match Person::select_by_tg_login(tg_login_from) {
        Some(x) => match x.role {
            PersonRole::Admin => (),
            _ => return String::from("Ошибка: доступно только администраторам."),
        },
        _ => return String::from("Ошибка: доступно только администраторам."),
    };

    let cmd = args.join(" ");
    let kwargs = util::parse_kwargs(cmd.as_str());

    let id: u32 = kwargs.get("id").unwrap_or(&String::new()).parse().unwrap_or(0);
    let tg_login: String = kwargs.get("tg_login").unwrap_or(&String::new()).to_string();
    let phone: String = kwargs.get("phone").unwrap_or(&String::new()).to_string();
    let email: String = kwargs.get("email").unwrap_or(&String::new()).to_string();
    let fio: String = kwargs.get("name").unwrap_or(&String::new()).to_string();

    let mut person: Person;
    if id != 0 {
        let mut stored_persons = Person::select_by_ids(vec![id].as_ref());
        if stored_persons.len() < 1 {
            return String::from("Нет пользователя с таким id");
        } else {
            person = stored_persons.get(0).unwrap().clone();
        }
    } else {
        person = Person {
            id, tg_login, phone, email, fio, role: PersonRole::User
        };
    }
    person.save();
    if person.id == 0 {
        return format!("Ошибка: не удалось сохранить пользователя: {}", person.to_string(&PersonRole::Admin));
    }
    return format!("Сохранено: {}", person.to_string(&PersonRole::Admin));
}

fn remove(args: &Vec<&str>) -> String {
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

fn info(args: &Vec<&str>, tg_login_from: &String) -> String {
    let from_role = match Person::select_by_tg_login(tg_login_from) {
        Some(x) => x.role,
        _ => PersonRole::Undefined,
    };

    let mut response = String::new();
    let mut person_ids = Vec::<u32>::new();


    for i in 1..args.len() {
        match args[i].parse::<u32>() {
            Ok(t) => person_ids.push(t),
            _ => response.push_str(format!("Ошибка: странный id пользователя: '{}'\n", args[i]).as_str()),
        };
    }
    let persons = Person::select_by_ids(&person_ids);
    for p in &persons {
        response.push_str(format!("{}\n", p.to_string(&from_role)).as_str());
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
    let arguments: Vec<&str> = msg.text.as_ref().unwrap().split(" ").collect();

    return match arguments[0] {
        "help" => help(""),
        "add" => add_or_update(arguments.as_ref(), &msg.from.as_ref().unwrap().username.unwrap_or(String::new())),
        "update" => add_or_update(arguments.as_ref(), &msg.from.as_ref().unwrap().username.unwrap_or(String::new())),
        "remove" => remove(arguments.as_ref()),
        "info" => info(arguments.as_ref(), &msg.from.as_ref().unwrap().username.unwrap_or(String::new())),
        "link_room" => link_room(arguments.as_ref()),
        "unlink_room" => unlink_room(arguments.as_ref()),
        _ => help("Unknown command")
    };
}