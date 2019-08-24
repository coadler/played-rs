extern crate eetf;
extern crate num;
extern crate redis;
extern crate tokio;

use eetf::{Map, Term};
use num::ToPrimitive;
use redis::Commands;
use std::io::Cursor;

#[allow(dead_code)]
#[derive(Debug)]
struct PlayedMessage {
    user: i64,
    game: String,
}

#[allow(unused_variables)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rc = redis::Client::open("redis://127.0.0.1:6380")?;
    let mut conn = rc.get_connection()?;

    loop {
        let cmd: (String, Vec<u8>) = conn.blpop("gateway:events:PRESENCE_UPDATE", 0)?;
        let (_, mut raw) = cmd;

        tokio::spawn(async move {
            let mut cmdd: Vec<u8> = vec![131];
            cmdd.append(&mut raw);
            let t: Term = Term::decode(Cursor::new(&mut cmdd)).unwrap();

            let m = match t {
                Term::Map(m) => m,
                t => {
                    println!("unknown type {}", t);
                    return;
                }
            };

            let mut pl = PlayedMessage {
                user: 0,
                game: String::from(""),
            };

            for i in m.entries {
                if let Term::Atom(k) = i.0 {
                    match k.name.as_ref() {
                        "user" => {
                            if let Term::Map(u) = i.1 {
                                pl.user = find_id(u);
                            }
                        }
                        "game" => {
                            pl.game = find_game(i.1);
                        }
                        _ => {}
                    }
                }
            }

            dbg!(&pl);
        });
    }
}

fn find_game(g: Term) -> String {
    if let Term::Map(g) = g {
        for i in g.entries {
            if let Term::Atom(k) = i.0 {
                if "name" == k.name {
                    match i.1 {
                        Term::Atom(game) => return game.name,
                        Term::Binary(game) => return String::from_utf8(game.bytes).unwrap(),
                        _ => println!("game type {}", &i.1),
                    }
                }
            }
        }
    }

    String::from("")
}

fn find_id(u: Map) -> i64 {
    for i in u.entries {
        if let Term::Atom(g) = i.0 {
            if g.name == "id" {
                if let Term::BigInteger(big) = i.1 {
                    if let Some(id) = big.value.to_i64() {
                        return id;
                    }
                }
            }
        }
    }

    0
}
