use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, TimeZone, Utc};
use foundationdb::*;
use futures::FutureExt;
use std::convert::TryInto;
use std::time::Duration;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let net = std::thread::spawn(|| {
        foundationdb::boot(|| {
            std::thread::park();
        });
    });

    tokio::time::delay_for(Duration::from_millis(500)).await;

    let db: Database = foundationdb::Database::default().expect("open fdb");

    let res = db
        .transact_boxed(
            (),
            |tx, _| read_exec(tx, b"105484726235607040").boxed(),
            TransactOption::default(),
        )
        .await?;
    dbg!(res);

    Ok(())
}

#[derive(Debug)]
pub struct Response {
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub games: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub dur: u32,
}

async fn read_exec<T: AsRef<[u8]>>(t: &foundationdb::Transaction, user: T) -> FdbResult<Response> {
    let games: Vec<Entry> = t
        .get_range(&fmt_user_range(user.as_ref()), 1, true)
        .await?
        .into_iter()
        .map(|v| {
            let (_, _, _, game): (Vec<u8>, u16, Vec<u8>, Vec<u8>) = tuple::unpack(v.key()).unwrap();
            Entry {
                name: String::from_utf8_lossy(&game).to_string(),
                dur: byteorder::LittleEndian::read_u64(v.value()) as u32,
            }
        })
        .collect();

    let first_seen = t
        .get(&fmt_first_seen_key(user.as_ref()), true)
        .await?
        .map(|v| bytes_to_date(&*v))
        .unwrap_or(Utc.timestamp(0, 0));

    let last_updated = t
        .get(&fmt_last_updated_key(user.as_ref()), true)
        .await?
        .map(|v| bytes_to_date(&*v))
        .unwrap_or(Utc.timestamp(0, 0));

    Ok(Response {
        first_seen,
        last_updated,
        games,
    })
}

fn bytes_to_date(raw: &[u8]) -> DateTime<Utc> {
    let secs = LittleEndian::read_u64(raw);
    Utc.timestamp(secs.try_into().unwrap(), 0)
}

const SUBSPACE_PREFIX: &[u8] = b"played";

enum Subspace {
    FirstSeen = 1,
    LastUpdated = 2,
    Current = 3,
    UserGame = 4,
}

fn fmt_first_seen_key(user: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::FirstSeen as u16))
        .pack(&user)
}

fn fmt_last_updated_key(user: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::LastUpdated as u16))
        .pack(&user)
}

fn fmt_current_game_key(user: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::Current as u16))
        .pack(&user)
}

fn fmt_user_game(user: &[u8], game: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::UserGame as u16))
        .pack(&(user, game))
}

#[allow(dead_code)]
fn fmt_user_range<'a>(user: &[u8]) -> RangeOption<'a> {
    RangeOption::from(
        tuple::Subspace::all()
            .subspace(&SUBSPACE_PREFIX)
            .subspace(&(Subspace::UserGame as u16))
            .subspace(&user)
            .range(),
    )
}
