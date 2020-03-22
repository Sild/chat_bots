use crate::db::model;
use crate::db::db_impl;
use std::ops::Add;

fn help(prefix: &str) -> String {
    return format!("{}
Usage:
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

    let mut rooms: Vec<model::Room> = Vec::new();
    let mut persons: Vec<model::Person> = Vec::new();
    let mut room_nums = String::new();
    for i in 1..args.len() {
        let room_num: i32 = args[i].parse().unwrap_or(-1);
        if room_num == -1 {
            response.push_str(format!("Ошибка: странный номер квартиры: '{}'\n", args[i]).as_str());
        } else {
            if room_nums.len() > 0 {
                room_nums.push_str(",");
            }
            room_nums.push_str(args[i]);
        }
    }
    for raw in db_impl::select(format!("select * from {} where num in ({}) order by num asc;", model::Room::tablename(), room_nums).as_str()).unwrap() {
        response.push_str(format!("{}\n", model::Room::from_vec(raw).to_string()).as_str());
    }
    println!("{}", response);
    return response;
}

fn find(args: &Vec<&str>) -> String {
//    person_db::add();
    return String::from("find");
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