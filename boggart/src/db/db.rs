use std::collections::HashMap;
use crate::db::{HouseID, ObjectID};
use crate::db::model::{Flat, Person};

pub enum DBError {
    FlatNotFound,
    PersonNotFound,
    Unknown,
}

pub trait DB {
    fn set_flat_count(&mut self, count: u32);
    fn upsert_person(&self, person: &Person);
    fn get_person_flats(&self, tg_login: &str) -> Vec<Flat>;
    fn get_flat_persons(&self, flat_id: ObjectID) -> Vec<Person>;
    fn link_person_flat(&self, person_id: ObjectID, flat_id: ObjectID);
    fn unlink_person_flat(&self, person_id: ObjectID, flat_id: ObjectID);
}

#[derive(Clone, Debug, Default)]
pub struct HouseDB {
    // for now flat_id == flat_number, but it can be changed in future
    flats: HashMap<ObjectID, Flat>,
    persons: HashMap<ObjectID, Person>,
    person_to_flat: HashMap<ObjectID, Vec<ObjectID>>,
    flat_to_person: HashMap<ObjectID, Vec<ObjectID>>,
}

#[derive(Clone, Debug, Default)]
pub struct BoggartDB {
    path: String,
    houses: HashMap<HouseID, HouseDB>,
}

impl DB for HouseDB {
    fn set_flat_count(&mut self, count: u32) {
        todo!()
    }
    fn add_person(&self, person: &Person) {
        todo!()
    }

    fn get_person_flats(&self, tg_login: &str) -> Vec<Flat> {
        todo!()
    }

    fn get_flat_persons(&self, flat_id: ObjectID) -> Vec<Person> {
        todo!()
    }

    fn link_person_flat(&self, person_id: ObjectID, flat_id: ObjectID) {
        todo!()
    }

    fn unlink_person_flat(&self, person_id: ObjectID, flat_id: ObjectID) {
        todo!()
    }
}

impl BoggartDB {
    pub fn new(path: &str) -> BoggartDB {
        todo!()
    }

    pub fn register_house(&self, house_id: &HouseID) {
        todo!()
    }

    pub fn get_house(&self, house_id: &HouseID) -> HouseDB {
        todo!()
    }
}