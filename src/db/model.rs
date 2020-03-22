pub enum  PersonRole {
    User,
    Admin,
    Undefined,
}

pub struct Person {
    pub id: i32,
    pub tg_login: String,
    pub email: String,
    pub fio: String,
    pub role: PersonRole,
}

impl Person {
    pub fn from_vec(fields: Vec<String>) -> Person {
        let role = match fields[4].as_str() {
            "User" => PersonRole::User,
            "Admin" => PersonRole::Admin,
            _ => PersonRole::Undefined,
        };

        return Person {
            id: fields[0].parse().unwrap(),
            tg_login: fields[1].to_string(),
            email: fields[2].to_string(),
            fio: fields[3].to_string(),
            role,
        }
    }

    pub fn from_fields(tg_login: &str, email: &str, fio: &str) -> Person {
        return Person {
            id: -1,
            tg_login: String::from(tg_login),
            email: String::from(email),
            fio: String::from(fio),
            role: PersonRole::User
        }
    }

    pub fn tablename() -> String {
        return String::from("person");
    }
}

pub struct Room {
    pub id: i32,
    pub num: i32,
    pub section: i32,
    pub floor: i32,
}
impl Room {
    pub fn from_vec(fields: Vec<String>) -> Room {
        return Room {
            id: fields[0].parse().unwrap(),
            num: fields[1].parse().unwrap(),
            section: fields[2].parse().unwrap(),
            floor: fields[3].parse().unwrap(),
        }
    }

    pub fn tablename() -> String {
        return String::from("room");
    }

    pub fn to_string(&self) -> String {
        return format!("квартира: {}, секция: {}, этаж: {}", self.num, self.section, self.floor);
    }
}

pub struct PersonRoom {
    pub id: i32,
    pub person_id: i32,
    pub room_id: i32,
}

impl PersonRoom {
    pub fn from_vec(fields: Vec<String>) -> PersonRoom {
        return PersonRoom {
            id: fields[0].parse().unwrap(),
            person_id: fields[1].parse().unwrap(),
            room_id: fields[2].parse().unwrap(),
        }
    }

    pub fn tablename() -> String {
        return String::from("person_to_room");
    }
}