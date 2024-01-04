extern crate dict;
use dict::DictIface;

use crate::db::model;
use crate::db::db_impl;
use crate::util;
use crate::bot_wrapper;



fn help(prefix: &str) -> String {
    return format!("{}
Использование:
/room help
/room info room_num1 room_num2 room_num3 ...
/room find section='' floor=''
    ", prefix);
}

fn info(args: &Vec<&str>) -> String {
    if args.len() < 3 {
        return help("Не достаточно аргументов: введите хотя бы 1 номер квартиры");
    }
    let mut response = String::new();

    let mut room_nums = Vec::<u32>::new();
    for i in 2..args.len() {
        match args[i].parse::<u32>() {
            Ok(t) => room_nums.push(t),
            _ => response.push_str(format!("Ошибка: странный номер квартиры: '{}'\n", args[i]).as_str()),
        };
    }
    let rooms = model::Flat::select_by_room_nums(room_nums.as_ref());
    let person_rooms = model::PersonRoom::select_by_room_ids(
        rooms.iter().map(|x| x.id).collect::<Vec<u32>>().as_ref()
    );
    let persons = model::Person::select_by_ids(
        person_rooms.iter().map(|x| x.person_id).collect::<Vec<u32>>().as_ref()
    );

    response.push_str(
        util::format_response_room_info(rooms.as_ref(), persons.as_ref(), person_rooms.as_ref()).as_str()
    );
    println!("{}", response);
    return response;
}

fn find(args: &Vec<&str>) -> String {
    let cmd = args.join(" ");
    let kwargs = util::parse_kwargs(cmd.as_str());

    let section: i32 = kwargs.get("section").unwrap_or(&String::new()).parse().unwrap_or(-1);
    let floor: i32 = kwargs.get("floor").unwrap_or(&String::new()).parse().unwrap_or(-1);

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

    let rooms: Vec<model::Flat> = db_impl::select(
        format!("select id, num, section, floor from {} where {} and {} order by num asc;"
                , model::Flat::tablename()
                , section_cond
                , floor_cond
        ).as_str()
    ).iter().map(|x| model::Flat::from_vec(x)).collect();

    let mut response = String::new();
    let mut oneline = String::from("список квартир: ");
    for r in rooms {
        response.push_str(format!("{}\n",r.to_string()).as_str());
        oneline.push_str(format!("{} ", r.num).as_str());
    }
    response.push_str(oneline.as_str());
    return response;
}

pub fn handle(msg: &bot_wrapper::Message) -> String {
    let arguments: Vec<&str> = msg.data.split(" ").collect();
    let command = arguments.get(1).unwrap_or(&"/help");

    return match *command {
        "help" => help(""),
        "info" => info(arguments.as_ref()),
        "find" => find(arguments.as_ref()),
        _ => help("Неизвесная команда")
    };
}