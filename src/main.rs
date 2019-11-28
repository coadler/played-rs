#![feature(async_closure)]
pub mod played;

extern crate foundationdb;
extern crate futures;
extern crate num;
extern crate redis;
extern crate tokio;
#[macro_use]
extern crate bitflags;
extern crate serde;
extern crate serde_eetf;
extern crate serde_repr;
extern crate ws;

use foundationdb::*;
use futures::future::*;
use played::Presence;
use ws::listen;

#[allow(dead_code)]
#[allow(unused_mut)]
#[allow(unused_variables)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = foundationdb::boot().expect("failed to init boot network");
    let db = foundationdb::Database::default().expect("failed to open fdb");

    listen("127.0.0.1:8080", |out| handler).expect("failed to handle ws");
    Ok(())
}

struct Played<'a> {
    db: &'a Database,
}

fn handler(msg: ws::Message) -> ws::Result<()> {
    let mut data: Vec<u8> = vec![131];
    data.append(msg.into_data().as_mut());
    let res: Presence = serde_eetf::from_bytes(&data).unwrap();
    dbg!(res);

    // tokio::spawn(process(db));

    Ok(())
}

async fn process(db: Database) {
    db.transact(
        (),
        |txn: &Transaction, ()| test(txn).boxed_local(),
        TransactOption::default(),
    )
    .await
    .unwrap();
}

async fn test(txn: &Transaction) -> FdbResult<()> {
    Ok(())
}
