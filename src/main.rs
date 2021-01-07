extern crate postgres;
extern crate serde;
extern crate serde_json;

mod db;

use postgres::{Connection, SslMode, ConnectParams, ConnectTarget, UserInfo};
use std::fmt::Debug;
use std::str::FromStr;
use std::fs::File;
use std::io::{Read, Error};


use serde::{Deserialize, Serialize};
use serde_json::Result;

const HELP: &'static str = "Usage: phonebook COMMAND [ARG]...
Commands:
    add NAME PHONE - create new record;
    del ID1 ID2... - delete record;
    edit ID        - edit record;
    show           - display all records;
    show STRING    - display records which contain a given substring in the name;
    help           - display this help.";

#[derive(Serialize, Deserialize, Debug)]
pub struct Params {
    host: String,
    port: String,
    ssl_mode: String,
    dbname: String,
    user: String,
    pass: String,
}

fn params() -> (ConnectParams, SslMode) {
    let mut contents = std::fs::read_to_string("params.json").expect("Unable to read file");

    let params: Params = serde_json::from_str(&contents).unwrap();

    let host = params.host;
    let port = params.port;
    let sslmode = params.ssl_mode;
    let dbname = params.dbname;
    let user = params.user;
    let pass = params.pass;
    
    let sslmode_ = match sslmode.as_ref() {
        "disable" => SslMode::None,
        "enable" => unimplemented!(),
        _ => panic!("Wrong sslmode"),
    };

    let params = ConnectParams {
        target: ConnectTarget::Tcp(host.to_owned()),
        port: Some(FromStr::from_str(&port).unwrap()),
        user: Some(UserInfo {
            user: user.to_owned(),
            password: Some(pass.to_owned()),
        }),
        database: Some(dbname.to_owned()),
        options: vec![],
    };
    (params, sslmode_)
}

struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>
}

fn main() {
    let (params, sslmode) = params();

    let conn =
        Connection::connect(
            "postgres://postgres:master@localhost",
            &SslMode::None)
            .unwrap();

    conn.execute(
        "
        DROP TABLE IF EXISTS person;
        ",
        &[])
        .unwrap();

    conn.execute(
        "
        CREATE TABLE person (
           id              SERIAL PRIMARY KEY,
           name            VARCHAR NOT NULL,
           data            BYTEA
         )",
        &[])
        .unwrap();

    let me = Person {
        id: 0,
        name: "Михаил".to_string(),
        data: None
    };

    conn.execute(
        "INSERT INTO person (name, data) VALUES ($1, $2)",
        &[&me.name, &me.data])
        .unwrap();

    let stmt = conn.prepare("SELECT id, name, data FROM person").unwrap();

    for row in stmt.query(&[]).unwrap() {
        let person = Person {
            id: row.get(0),
            name: row.get(1),
            data: row.get(2)
        };
        println!("Нашли человека: {}", person.name);
    }
}

fn parse() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1) {
        Some(text) => {
            match text.as_ref() {
                "add" => {
                    if args.len() != 4 {
                        panic!("Usage: phonebook add NAME PHONE");
                    }
                    let r = db::insert(db, &args[2], &args[3])
                        .unwrap();
                    println!("{} rows affected", r);
                },
                "del" => {
                    if args.len() < 3 {
                        panic!("Usage: phonebook del ID...");
                    }
                    let ids: Vec<i32> = args[2..].iter()
                        .map(|s| s.parse().unwrap())
                        .collect();

                    db::remove(db, &ids)
                        .unwrap();
                },
                "edit" => {
                    if args.len() != 5 {
                        panic!("Usage: phonebook edit ID NAME PHONE");
                    }
                    let id = args[2].parse().unwrap();
                    db::update(db, id, &args[3], &args[4])
                        .unwrap();
                },
                "show" => {
                    if args.len() > 3 {
                        panic!("Usage: phonebook show [SUBSTRING]");
                    }
                    let s;
                    if args.len() == 3 {
                        s = args.get(2);
                    } else {
                        s = None;
                    }
                    let r = db::show(db, s.as_ref().map(|s| &s[..])).unwrap();
                    db::format(&r);
                },
                "help" => {
                    println!("{}", HELP);
                },
                command @ _  => panic!(
                    format!("Invalid command: {}", command))
            }
        }
        None => panic!("No command supplied"),
    }
}