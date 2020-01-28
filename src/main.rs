#![feature(async_closure)]
pub mod played;
#[macro_use]
extern crate bitflags;

use byteorder::{ByteOrder, LittleEndian};
use chrono::Utc;
use foundationdb::*;
use futures::future::*;
use played::Presence;
use std::error::Error;
use std::ptr;
use std::sync::Arc;

const global_fdb: *const Database = ptr::null();
const global_rdb: *const i32 = ptr::null();

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        foundationdb::boot().expect("failed to init boot network");
        let db = foundationdb::Database::default().expect("failed to open fdb");
        *global_fdb = db;

        ws::listen("127.0.0.1:8080", |out| handler()).expect("failed to handle ws");
    }
    Ok(())
}

fn handler() -> Box<dyn Fn(ws::Message) -> ws::Result<()>> {
    Box::new(move |msg: ws::Message| -> ws::Result<()> {
        let mut data: Vec<u8> = vec![131];
        data.append(msg.into_data().as_mut());
        let res: Presence = serde_eetf::from_bytes(&data).unwrap();
        dbg!(&res);

        unsafe {
            tokio::spawn(process(res));
        }
        Ok(())
    })
}

async fn process(pres: Presence) {
    unsafe {
        ptr::read(global_fdb.as_ref().unwrap())
            .transact_boxed(
                pres,
                |txn: &Transaction, pres| exec(txn, pres).boxed(),
                TransactOption::default(),
            )
            .await
            .unwrap();
    }
}

async fn exec(txn: &foundationdb::Transaction, pres: &Presence) -> FdbResult<()> {
    let now = Utc::now().timestamp() as u64;
    let mut now_raw = [0; 4];
    LittleEndian::write_u64(&mut now_raw, now);

    Ok(())
}

// HACK
// since we don't have FDB directory support this is the hardcoded dir prefix in prod
static SUBSPACE_PREFIX: [u8; 2] = [0x15, 0x34];

#[allow(dead_code)]
fn fmt_first_seen_key(user: &String) -> Vec<u8> {
    tuple::Subspace::from_bytes(&SUBSPACE_PREFIX)
        .subspace(&String::from("first-seen"))
        .pack(user)
}

#[allow(dead_code)]
fn fmt_last_updated_key(user: &String) -> Vec<u8> {
    tuple::Subspace::from_bytes(&SUBSPACE_PREFIX)
        .subspace(&String::from("last_updated"))
        .pack(user)
}

#[allow(dead_code)]
fn fmt_current_game_key(user: &String) -> Vec<u8> {
    tuple::Subspace::from_bytes(&SUBSPACE_PREFIX)
        .subspace(&String::from("current"))
        .pack(user)
}

#[allow(dead_code)]
fn fmt_played_user_game(user: &String, game: &String) -> Vec<u8> {
    tuple::Subspace::from_bytes(&SUBSPACE_PREFIX)
        .subspace(&String::from("played"))
        .subspace(user)
        .pack(game)
}

#[allow(dead_code)]
fn fmt_played_user_range(user: &String) -> RangeOption {
    RangeOption::from(
        tuple::Subspace::from_bytes(&SUBSPACE_PREFIX)
            .subspace(&String::from("played"))
            .subspace(user)
            .range(),
    )
}

#[allow(dead_code)]
fn fmt_whitelist_key(user: &String) -> String {
    format!("played:whitelist:{}", user)
}
