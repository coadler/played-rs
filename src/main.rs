#![feature(async_closure)]
pub mod played;
#[macro_use]
extern crate bitflags;

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
    let network = foundationdb::boot().expect("failed to init boot network");
    let db = Arc::new(foundationdb::Database::default().expect("failed to open fdb"));

    ws::listen("127.0.0.1:8080", |out| handler(Arc::clone(&db))).expect("failed to handle ws");
    Ok(())
}

fn handler(db: Arc<foundationdb::Database>) -> Box<dyn Fn(ws::Message) -> ws::Result<()>> {
    Box::new(move |msg: ws::Message| -> ws::Result<()> {
        let mut data: Vec<u8> = vec![131];
        data.append(msg.into_data().as_mut());
        let res: Presence = serde_eetf::from_bytes(&data).unwrap();
        dbg!(res);

        tokio::spawn(process(Arc::clone(&db)));
        Ok(())
    })
}

async fn process(db: Arc<Database>) {
    db.transact_boxed(
        (),
        |txn: &Transaction, ()| test(txn).boxed(),
        TransactOption::default(),
    )
    .await
    .unwrap();
}

async fn test(txn: &foundationdb::Transaction) -> FdbResult<()> {
    Ok(())
}
