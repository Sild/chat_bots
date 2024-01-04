extern crate sqlite;

use crate::config;

pub fn exec(query: &str) {
    println!("exec query: '{}'", query);
    let connection = sqlite::open(std::path::Path::new(config::DB_PATH)).unwrap();
    match connection.execute(query) {
        Ok(_x) => println!("Executed successfully"),
        _ => println!("Fail to execute query"),
    };
}

pub fn select(query: &str) -> Vec<Vec<String>> {
    println!("select query: '{}'", query);
    let connection = sqlite::open(std::path::Path::new(config::DB_PATH)).unwrap();
    let mut statement = connection.prepare(query).unwrap();
    let mut res: Vec<Vec<String>> = Vec::new();
    while let sqlite::State::Row = statement.next().unwrap() {
        let mut raw: Vec<String> = Vec::new();
        for i in 0..statement.count() {
            raw.push(statement.read::<String>(i).unwrap_or(String::new()));
        }
        res.push(raw);
    };
    println!("items in result={}", res.len());
    return res;
}
