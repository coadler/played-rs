#![feature(async_closure)]
pub mod played;
#[macro_use]
extern crate bitflags;

use byteorder::{ByteOrder, LittleEndian};
use chrono::Utc;
use foundationdb::*;
use futures::future::*;
use futures::{stream::TryStreamExt, StreamExt};
use played::Presence;
use std::cell::UnsafeCell;
use std::env;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

struct Server {
    fdb: UnsafeCell<Database>,
}

unsafe impl Sync for Server {}
unsafe impl Send for Server {}

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    foundationdb::boot().expect("failed to init boot network");
    let db: Database = foundationdb::Database::default().expect("failed to open fdb");

    let p: *const Server = &Server {
        fdb: UnsafeCell::new(db),
    };

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        unsafe {
            tokio::spawn((*p).accept(stream));
        }
    }

    Ok(())
}

impl Server {
    async unsafe fn process(&self, pres: Presence) {
        let db: *const Database = self.fdb.get();
        (*db)
            .transact_boxed(
                pres,
                |txn: &Transaction, pres| exec(txn, pres).boxed(),
                TransactOption::default(),
            )
            .await
            .unwrap()
    }

    async unsafe fn accept(&self, stream: TcpStream) {
        let mut buf: Vec<u8> = vec![];
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        ws_stream
            .try_for_each(|msg| {
                buf.truncate(0);
                buf.push(131);
                buf.append(msg.into_data().as_mut());
                let res: Presence = serde_eetf::from_bytes(&buf).unwrap();
                tokio::spawn(self.process(res));
                ok(())
            })
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
