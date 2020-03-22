extern crate sqlite;

use crate::config;

pub fn exec(query: &str) -> Result<bool, String> {
    println!("exec query: {}", query);
    let connection = sqlite::open(std::path::Path::new(config::DB_PATH)).unwrap();
    return match connection.execute(query) {
        Ok(x) => Ok(true),
        _ => Err(String::from("fail to execute db query")),
    };
}

pub fn select(query: &str) -> Result<Vec<Vec<String>>, String> {
    println!("select query: {}", query);
    let connection = sqlite::open(std::path::Path::new(config::DB_PATH)).unwrap();
    let mut statement = connection.prepare(query).unwrap();

    let mut res: Vec<Vec<String>> = Vec::new();
    while let sqlite::State::Row = statement.next().unwrap() {
        let mut raw: Vec<String> = Vec::new();
        for i in 0..statement.count() {
            raw.push(statement.read::<String>(i).unwrap());
        }
        res.push(raw);
    };
    return Result::Ok(res);
}
