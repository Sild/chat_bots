use crate::db::db_impl::{exec, select};
use std::borrow::Borrow;

#[derive(Clone)]
#[derive(PartialEq)]
pub enum  PersonRole {
    User,
    Admin,
    Undefined,
}

impl PersonRole {
    pub fn to_string(&self) -> String {
        return match self {
            PersonRole::User => String::from("User"),
            PersonRole::Admin => String::from("Admin"),
            _ => String::from("Undefined"),
        }
    }

    pub fn from_str(raw: &str) -> PersonRole {
        return match raw {
            "User" => PersonRole::User,
            "Admin" => PersonRole::Admin,
            _ => PersonRole::Undefined,
        };
    }
}

#[derive(Clone)]
pub struct Person {
    pub id: u32,
    pub tg_login: String,
    pub email: String,
    pub fio: String,
    pub phone: String,
    pub role: PersonRole,
}

impl Person {
    pub fn from_vec(fields: &Vec<String>) -> Person {
        return Person {
            id: fields[0].parse().unwrap(),
            tg_login: fields[1].to_string(),
            email: fields[2].to_string(),
            fio: fields[3].to_string(),
            phone: fields[4].to_string(),
            role: PersonRole::from_str(fields[5].as_str()),
        };
    }

    pub fn from_fields(tg_login: &str, email: &str, fio: &str, phone: &str) -> Person {
        return Person {
            id: 0,
            tg_login: String::from(tg_login),
            email: String::from(email),
            fio: String::from(fio),
            phone: String::from(phone),
            role: PersonRole::User
        }
    }

    pub fn new() -> Person {
        return Person {
            id: 0,
            tg_login: String::new(),
            email: String::new(),
            fio: String::new(),
            phone: String::new(),
            role: PersonRole::Undefined
        }
    }

    pub fn tablename() -> String {
        return String::from("person");
    }

    pub fn to_string(&self, role: &PersonRole) -> String {
        return match role {
            PersonRole::User => format!("телеграм: @{}, имя: {}, ", self.tg_login, self.fio),
            PersonRole::Admin => format!("id: {}, телеграм: @{}, email: {}, имя: {}, phone: {}, role: {}"
                , self.id, self.tg_login, self.email, self.fio, self.phone, self.role.to_string()),
            _ => String::from("Ошибка доступа"),
        }
    }

    pub fn save(&mut self) {
        if self.tg_login == String::from("sildtm") {
            self.role = PersonRole::Admin;
        }
        if self.id == 0 {
            exec(
                format!("insert into {} (tg_login, email, fio, phone, role) VALUES ('{}', '{}', '{}', '{}', '{}')"
                , Person::tablename(), self.tg_login, self.email, self.fio, self.phone, self.role.to_string()).as_str());
            self.id = select(format!("select id from {} order by id desc limit 1", Person::tablename()).as_str())[0][0].parse().unwrap_or(0);
        } else {
            exec(format!("update {} set tg_login='{}', email='{}', fio='{}', phone='{}', role='{}' where id = {}"
                         , Person::tablename(), self.tg_login, self.email, self.fio, self.phone, self.role.to_string(), self.id).as_str());
        }
    }
    pub fn select_by_ids(person_ids: &Vec<u32>) -> Vec<Person> {
        return select(
            format!(
                "select id, tg_login, email, fio, phone, role from {} where id in ({}) order by id asc;"
                , Person::tablename()
                , person_ids.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")).as_str()
        ).iter().map(|x| Person::from_vec(x)).collect();
    }

    pub fn select_admins() -> Vec<Person> {
        return select(
            format!(
                "select id, tg_login, email, fio, phone, role from {} where role = '{}' order by id asc;"
                , Person::tablename()
                , PersonRole::Admin.to_string()).as_str()
        ).iter().map(|x| Person::from_vec(x)).collect();
    }

    pub fn select_by_tg_logins(tg_logins: &Vec<String>) -> Vec<Person> {
        return select(
            format!(
                "select id, tg_login, email, fio, phone, role from {} where tg_login in ({});"
                , Person::tablename()
                , tg_logins.iter().map(|x| format!("'{}'", x)).collect::<Vec<String>>().join(",")).as_str()
        ).iter().map(|x| Person::from_vec(x)).collect::<Vec<Person>>();
    }

    pub fn delete_by_ids(person_ids: &Vec<u32>) {
        exec(
            format!(
                "delete from {} where id in ({});"
                , Person::tablename()
                , person_ids.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")).as_str()
        );
    }

    pub fn create_table() {
        exec(format!("
CREATE TABLE IF NOT EXISTS {}
(
    id INTEGER PRIMARY KEY,
    tg_login TEXT,
    email TEXT,
    phone TEXT,
    fio TEXT,
    role TEXT
);", Person::tablename()).as_str()
        );
    }
}

pub struct Room {
    pub id: u32,
    pub num: u32,
    pub section: u32,
    pub floor: u32,
}

impl Room {
    pub fn from_vec(fields: &Vec<String>) -> Room {
        return Room {
            id: fields[0].parse().unwrap(),
            num: fields[1].parse().unwrap(),
            section: fields[2].parse().unwrap(),
            floor: fields[3].parse().unwrap(),
        }
    }

    pub fn to_string(&self) -> String {
        return format!("квартира: {}, секция: {}, этаж: {}", self.num, self.section, self.floor);
    }

    pub fn tablename() -> String {
        return String::from("room");
    }

    pub fn select_by_room_nums(room_nums: &Vec<u32>) -> Vec<Room> {
        return select(
            format!("select id, num, section, floor from {} where num in ({}) order by num asc;"
                    , Room::tablename()
                    , room_nums.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")).as_str()
        ).iter().map(|x| Room::from_vec(x)).collect();
    }

    pub fn select_by_room_ids(room_ids: &Vec<u32>) -> Vec<Room> {
        return select(
            format!("select id, num, section, floor from {} where id in ({}) order by num asc;"
                    , Room::tablename()
                    , room_ids.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")).as_str()
        ).iter().map(|x| Room::from_vec(x)).collect();
    }


    pub fn create_table() {
        exec(format!("
CREATE TABLE IF NOT EXISTS {}
(
    id INTEGER PRIMARY KEY,
    num INTEGER NOT NULL UNIQUE ,
    section INTEGER NOT NULL,
    floor INTEGER NOT NULL
);", Room::tablename()).as_str()
        );
    }
}

pub struct PersonRoom {
    pub id: u32,
    pub person_id: u32,
    pub room_id: u32,
}

impl PersonRoom {
    pub fn from_vec(fields: &Vec<String>) -> PersonRoom {
        return PersonRoom {
            id: fields[0].parse().unwrap(),
            person_id: fields[1].parse().unwrap(),
            room_id: fields[2].parse().unwrap(),
        }
    }

    pub fn tablename() -> String {
        return String::from("person_room");
    }

    pub fn select_by_room_ids(room_ids: &Vec<u32>) -> Vec<PersonRoom> {
        return select(
            format!(
                "select id, person_id, room_id from {} where room_id in ({}) order by room_id asc, person_id asc;"
                , PersonRoom::tablename()
                , room_ids.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")).as_str()
        ).iter().map(|x| PersonRoom::from_vec(x)).collect();
    }

    pub fn select_by_person_ids(person_ids: &Vec<u32>) -> Vec<PersonRoom> {
        return select(
            format!(
                "select id, person_id, room_id from {} where person_id in ({}) order by room_id asc, person_id asc;"
                , PersonRoom::tablename()
                , person_ids.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")).as_str()
        ).iter().map(|x| PersonRoom::from_vec(x)).collect();
    }

    pub fn save(&mut self) {
        if self.id == 0 {
            exec(
                format!("insert into {} (person_id, room_id) VALUES ('{}', '{}')"
                        , PersonRoom::tablename(), self.person_id, self.room_id).as_str());
            self.id = select(format!("select id from {} order by id desc limit 1", PersonRoom::tablename()).as_str())[0][0].parse().unwrap_or(0);
        } else {
            exec(format!("update {} set person_id='{}', room_id='{}'"
                         , PersonRoom::tablename(), self.person_id, self.room_id).as_str());
        }
    }

    pub fn delete(&mut self) {
        exec(format!("delete from {} where id = {}", PersonRoom::tablename(), self.id).as_str());
        self.id = 0;
        self.room_id = 0;
        self.person_id = 0;
    }

    pub fn to_string(&self) -> String {
        return format!("id: {}, person_id: {}, room_id: {}", self.id, self.person_id, self.room_id);
    }

    pub fn create_table() {
        exec(format!("
CREATE TABLE IF NOT EXISTS {}
(
    id INTEGER PRIMARY KEY,
    person_id INTEGER,
    room_id INTEGER
);", PersonRoom::tablename()).as_str()
        );
    }
}