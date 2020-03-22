extern crate regex;

use crate::db::model;
use crate::db::db_impl;

fn help(prefix: &str) -> String {
    return format!("{}
Использование:
/room help
/room info room_num1 room_num2 room_num3 ...
/room find section='' floor=''
    ", prefix);
}

fn info(args: &Vec<&str>) -> String {
    if args.len() < 2 {
        return help("Не достаточно аргументов: введите хотя бы 1 номер квартиры");
    }
    let mut response = String::new();

    let mut room_nums = String::new();
    for i in 1..args.len() {
        match args[i].parse::<u32>() {
            Ok(_t) => {
                if room_nums.len() > 0 { room_nums.push(',')}
                room_nums.push_str(args[i])
            },
            _ => response.push_str(format!("Ошибка: странный номер квартиры: '{}'\n", args[i]).as_str()),
        };
    }
    let rooms: Vec<model::Room> = db_impl::select(
        format!("select id, num, section, floor from {} where num in ({}) order by num asc;", model::Room::tablename(), room_nums).as_str()
    ).iter().map(|x| model::Room::from_vec(x)).collect();

    let person_rooms: Vec<model::PersonRoom> = db_impl::select(
        format!(
            "select id, person_id, room_id from {} where room_id in ({}) order by room_id asc, person_id asc;"
            , model::PersonRoom::tablename()
            , rooms.iter().map(|x| x.id.to_string()).collect::<Vec<String>>().join(",")).as_str()
    ).iter().map(|x| model::PersonRoom::from_vec(x)).collect();

    let persons: Vec<model::Person> = db_impl::select(
        format!(
            "select id, tg_login, email, fio, phone, role from {} where id in ({}) order by id asc;"
            , model::Person::tablename()
            , person_rooms.iter().map(|x| x.person_id.to_string()).collect::<Vec<String>>().join(",")).as_str()
    ).iter().map(|x| model::Person::from_vec(x)).collect();

    for r in &rooms {
        response.push_str(format!("{}\n",r.to_string()).as_str());
        for pr in &person_rooms {
            if pr.room_id != r.id {
                continue;
            }
            for p in &persons {
                if p.id == pr.person_id {
                    response.push_str(format!("{}\n", p.to_string()).as_str());
                }
            }
        }
    }
    println!("{}", response);
    return response;
}

fn find(args: &Vec<&str>) -> String {
    let cmd = args.join(" ");
    let section_regexp = regex::Regex::new(r"^.*section=(\d*).*$").unwrap();
    let floor_regexp = regex::Regex::new(r"^.*floor=(\d*).*$").unwrap();

    let section = match section_regexp.captures(cmd.as_str()) {
        Some(x) => x.get(1).map_or("", |m| m.as_str()),
        _ => "",
    };
    let section: i32 = section.parse().unwrap_or(-1);

    let floor = match floor_regexp.captures(cmd.as_str()) {
        Some(x) => x.get(1).map_or("", |m| m.as_str()),
        _ => "",
    };
    let floor: i32 = floor.parse().unwrap_or(-1);

    if section == -1 && floor == -1 {
        return help("Запрос на все квартиры запрещен.\nВведите что-ннибудь для фильтрации.");
    }

    let section_cond = match section {
        -1 => String::from("1"),
        _ => format!("section = {}", section),
    };
    let floor_cond = match floor {
        -1 => String::from("1"),
        _ => format!("floor = {}", floor),
    };

    let rooms: Vec<model::Room> = db_impl::select(
        format!("select id, num, section, floor from {} where {} and {} order by num asc;"
                , model::Room::tablename()
                , section_cond
                , floor_cond
        ).as_str()
    ).iter().map(|x| model::Room::from_vec(x)).collect();

    let mut response = String::new();
    let mut oneline = String::from("список квартир: ");
    for r in rooms {
        response.push_str(format!("{}\n",r.to_string()).as_str());
        oneline.push_str(format!("{} ", r.num).as_str());
    }
    response.push_str(oneline.as_str());
    return response;
}

pub fn handle(msg: &telebot::objects::Message) -> String {
    let arguments: Vec<&str> = msg.text.as_ref().unwrap().split(" ").collect();
    return match arguments[0] {
        "help" => help(""),
        "info" => info(arguments.as_ref()),
        "find" => find(arguments.as_ref()),
        _ => help("Unknown command")
    };
}