#![feature(async_await)]

extern crate eetf;
extern crate redis;
extern crate tokio;


#[allow(unused_imports)]
use eetf::{Atom, Map, Term};
use redis::Commands;
use std::error::Error;
use std::io::Cursor;

#[allow(dead_code)]
struct PlayedMessage {
    user: i64,
    game: String,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rc = redis::Client::open("redis://127.0.0.1:6380")?;
    let mut conn = rc.get_connection()?;

    loop {
        let cmd: (String, Vec<u8>) = conn.blpop("gateway:events:PRESENCE_UPDATE", 0)?;
        let (_, mut raw) = cmd;

        tokio::spawn(async move {
            let mut cmdd: Vec<u8> = vec![131];
            cmdd.append(&mut raw);
            Term::decode(Cursor::new(&mut cmdd)).unwrap();
        });

    }
}
