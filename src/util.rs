extern crate dict;

use dict::{Dict, DictIface};
use crate::db::model;
use crate::db::model::PersonRole;

pub fn parse_kwargs(text: &str) -> Dict::<String> {
    let mut kwargs = Dict::<String>::new();

    let words: Vec<&str> = text.split(" ").collect();
    for &w in words.iter() {
        let kw: Vec<&str> = w.split("=").collect();
        if kw.len() >= 2 {
            kwargs.add(kw[0].to_string(), kw[1].to_string());
        }
    }
    return kwargs;
}

pub fn format_response_room_info(rooms: &Vec<model::Room>, persons: &Vec<model::Person>, person_rooms: &Vec<model::PersonRoom>) -> String {
    let mut response = String::new();
    for r in rooms {
        response.push_str(format!("{}\n",r.to_string()).as_str());
        for pr in person_rooms {
            if pr.room_id != r.id {
                continue;
            }
            for p in persons {
                if p.id == pr.person_id {
                    response.push_str(format!("    {}\n", p.to_string(&PersonRole::User)).as_str());
                }
            }
        }
    }
    return response;
}

pub fn format_response_person_info(persons: &Vec<model::Person>, rooms: &Vec<model::Room>, person_rooms: &Vec<model::PersonRoom>, role: &PersonRole) -> String {
    let mut response = String::new();
    for p in persons {
        response.push_str(format!("{}\n",p.to_string(role)).as_str());
        for pr in person_rooms {
            if pr.person_id != p.id {
                continue;
            }
            for r in rooms {
                if r.id == pr.room_id {
                    response.push_str(format!("    {}\n", r.to_string()).as_str());
                }
            }
        }
    }
    return response;
}

pub fn user_exists(tg_login_from: &str) {

}