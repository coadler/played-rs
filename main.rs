extern crate redis;

use eetf::{Atom, Map, Term};
use redis::Commands;
use std::error::Error;
use std::io::Cursor;

#[allow(dead_code)]
struct PlayedMessage {
    user: i64,
    game: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let rc = redis::Client::open("redis://127.0.0.1")?;
    let mut conn = rc.get_connection()?;

    loop {
        let cmd: Vec<u8> = conn.blpop("hello", 0)?;
        let term = Term::decode(Cursor::new(&cmd))?;
        println!("{}", term)
    }
}
