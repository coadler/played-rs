use crate::helpers::bytes_to_date;
use crate::keys::*;
use crate::Runner;
use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, TimeZone, Utc};
use foundationdb::{future::FdbValue, tuple, FdbResult, TransactOption};
use futures::prelude::*;

#[derive(Debug)]
pub struct Response {
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub games: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    /// seconds.
    pub dur: u32,
}

impl Runner {
    pub async fn read<T: AsRef<str>>(&self, user: T) -> Result<Response> {
        Ok(self
            .fdb
            .transact_boxed(
                user.as_ref().as_bytes(),
                |tx, usr| read_exec(tx, usr).boxed(),
                TransactOption::default(),
            )
            .await?)
    }
}

#[inline]
async fn read_exec<T: AsRef<[u8]>>(t: &foundationdb::Transaction, user: T) -> FdbResult<Response> {
    &dbg!(&fmt_user_range(user.as_ref()));
    let games: Vec<Entry> = t
        .get_range(&fmt_user_range(user.as_ref()), 1, true)
        .await?
        .into_iter()
        .map(|v| v.into())
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

impl From<FdbValue> for Entry {
    fn from(v: FdbValue) -> Self {
        let (_, _, _, game): UserGameKey = tuple::unpack(v.key()).unwrap();
        Entry {
            name: String::from_utf8_lossy(&game).to_string(),
            dur: LittleEndian::read_u64(v.value()) as u32,
        }
    }
}
