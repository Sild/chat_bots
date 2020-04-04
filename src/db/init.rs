use std::fs;

use crate::config;

use super::db_impl::exec;
use super::model;
use crate::db::model::{Person, PersonRole};

pub fn init_db() {
    let file_exists =  match fs::metadata(config::DB_PATH) {
        Ok(meta) => meta.is_file(),
        _ => false,
    };
    if file_exists {
        return;
    }

    model::Room::create_table();
    model::Person::create_table();
    model::PersonRoom::create_table();
    fill_room_table();
    let mut admin = Person {
        id: 0,
        tg_login: String::from("sildtm"),
        email: String::from("sildtm@icloud.com"),
        fio: String::from("Dmitry"),
        phone: String::new(),
        role: PersonRole::Admin,
    };
    admin.save();
//    fill_person_room_mock();
}

fn fill_room_table() {
    let mut room_num = 1;
    let mut query = format!("INSERT INTO {} (num, section, floor) VALUES ", model::Room::tablename());
    for x in 2..15 {
        for _i in 0..6 {
            query.push_str(format!("({}, {}, {}), ", room_num, 1, x).as_str());
            room_num += 1;
        }
    }

    for x in 2..7 {
        for _i in 0..10 {
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
    exec(query.as_str());
}

fn fill_person_room_mock() {
    let query = String::from("
INSERT INTO person (tg_login, fio, phone) VALUES
('sildtm', 'Kordd', '89959979747'), ('sildtm2', 'Kordd2', '9999');
    ");
     exec(query.as_str());

    let query = String::from("
INSERT INTO person_room (person_id, room_id) VALUES
(1, 2), (2, 3), (2, 4), (1, 3);
    ");
    exec(query.as_str());
}
