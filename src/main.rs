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
use std::error::Error;
use std::sync::Arc;

#[allow(dead_code)]
#[allow(unused_mut)]
#[allow(unused_variables)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut played = Played::new();

    ws::listen("127.0.0.1:8080", |out| played.clone().handler()).expect("failed to handle ws");
    Ok(())
}

struct Played {
    db: Database,
}

impl Played {
    fn new() -> Arc<Played> {
        foundationdb::boot().expect("failed to init boot network");
        let db = foundationdb::Database::default().expect("failed to open fdb");

        Arc::new(Played { db })
    }

    async fn process(&self, p: Presence) {
        self.db
            .transact(
                (),
                |txn: &Transaction, ()| test(txn).boxed_local(),
                TransactOption::default(),
            )
            .await
            .unwrap();
    }

    fn handler(&self) -> Box<dyn Fn(ws::Message) -> Result<(), ws::Error>> {
        Box::new(|msg| {
            let mut data: Vec<u8> = vec![131];
            data.append(msg.into_data().as_mut());
            let res: Presence = serde_eetf::from_bytes(&data).unwrap();
            dbg!(res);

            let f = self.process(res).boxed();
            tokio::spawn(f);
            Ok(())
        })
    }
}

async fn test(txn: &Transaction) -> FdbResult<()> {
    Ok(())
}
