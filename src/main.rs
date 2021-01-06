extern crate postgres;
extern crate serde;
extern crate serde_json;

use postgres::{Connection, SslMode, ConnectParams, ConnectTarget, UserInfo};
use std::fmt::Debug;
use std::str::FromStr;
use std::fs::File;
use std::io::{Read, Error};


use serde::{Deserialize, Serialize};
use serde_json::Result;

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
    let mut file = File::open("params.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

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
    params();

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