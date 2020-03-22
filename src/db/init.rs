use std::fs;

use super::db_impl::exec;
use crate::config;
use std::ops::Add;

use super::model;


fn create_person_table() -> Result<bool, String> {
    let query = String::from("
CREATE TABLE IF NOT EXISTS person
(
    id INTEGER PRIMARY KEY,
    tg_login TEXT,
    phone TEXT,
    email TEXT,
    fio TEXT,
    role TEXT
);
    ");
    return exec(query.as_str());
}

fn create_room_table() -> Result<bool, String>{
    let query = String::from("
CREATE TABLE IF NOT EXISTS room
(
    id INTEGER PRIMARY KEY,
    num INTEGER NOT NULL UNIQUE ,
    section INTEGER NOT NULL,
    floor INTEGER NOT NULL
);
    ");
    return exec(query.as_str());
}

fn fill_room_table() -> Result<bool, String>  {
    let mut room_num = 1;
    let mut query = String::from("INSERT INTO room (num, section, floor) VALUES ");
    for x in 2..15 {
        for i in 0..6 {
            query.push_str(format!("({}, {}, {}), ", room_num, 1, x).as_str());
            room_num += 1;
        }
    }

    for x in 2..7 {
        for i in 0..10 {
            query.push_str(format!("({}, {}, {}), ", room_num, 2, x).as_str());
            room_num += 1;
        }
    }

    for x in 2..9 {
        for i in 0..7 {
            if x == 8 && i == 6 {
                query.push_str(format!("({}, {}, {}); ", room_num, 3, x).as_str());
            } else {
                query.push_str(format!("({}, {}, {}), ", room_num, 3, x).as_str())
            }
            room_num += 1;
        }
    }
    return exec(query.as_str());
}

fn create_person_room_table() -> Result<bool, String> {
    let query = String::from(format!("
CREATE TABLE IF NOT EXISTS {}
(
    id INTEGER PRIMARY KEY,
    person_id INTEGER,
    room_id INTEGER
);
    ", model::Person::tablename()));
    return exec(query.as_str());
}


pub fn init_db() {
    let file_exists =  match fs::metadata(config::DB_PATH) {
        Ok(meta) => meta.is_file(),
         _ => false,
    };
    if file_exists {
        return;
    }

    create_person_table();
    create_room_table();
    fill_room_table();
    create_person_room_table();
}