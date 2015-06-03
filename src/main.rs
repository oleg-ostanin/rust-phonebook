extern crate ini;
extern crate postgres;

use ini::Ini;
use postgres::{Connection, ConnectParams, ConnectTarget, SslMode, UserInfo};

use std::str::FromStr;

fn params() -> (ConnectParams, SslMode) {
    let conf = Ini::load_from_file(".phonebookrc").unwrap();
    let general = conf.general_section();

    let host = general.get("host").unwrap();
    let port = general.get("port").unwrap();
    let sslmode = general.get("sslmode").unwrap();
    let dbname = general.get("dbname").unwrap();
    let user = general.get("user").unwrap();
    let pass = general.get("pass").unwrap();

    let s = match sslmode.as_ref() {
        "disable" => SslMode::None,
        "enable" => unimplemented!(),
        _ => panic!("Wrong sslmode"),
    };

    (ConnectParams {
        target: ConnectTarget::Tcp(host.to_owned()),
        port: Some(FromStr::from_str(port).ok().unwrap()),
        user: Some(UserInfo {
            user: user.to_owned(),
            password: Some(pass.to_owned()),
        }),
        database: Some(dbname.to_owned()),
        options: vec![],
    }, s)
}

fn main() {
    let (params, sslmode) = params();
    let db = Connection::connect(params, &sslmode).unwrap();

    db.execute(
        concat!(r#"CREATE TABLE IF NOT EXISTS phonebook"#,
                r#"("id" SERIAL PRIMARY KEY, "name" varchar(50),"#,
                r#" "phone" varchar(100))"#,
                ),
        &[]
            ).unwrap();

    let mut args = std::env::args();
    match args.nth(1) {
        Some(text) => {
            match text.as_ref() {
                "add" => {
                    if args.len() != 4 {
                        panic!("Usage: phonebook add NAME PHONE")
                    };
                    let r = insert(
                        db,
                        &args.nth(2).unwrap(),
                        &args.nth(3).unwrap()
                            ).unwrap();
                    println!("{} rows affected", r);
                },
                "del" => {},
                "edit" => {},
                "show" => {},
                "help" => {},
                command @ _  => panic!(
                    format!("Invalid command: {}", command))
            }
        }
        None => panic!("No command supplied"),
    }
}

fn insert(db: Connection, name: &str, phone: &str) -> postgres::Result<u64> {
    Ok(0)
}
