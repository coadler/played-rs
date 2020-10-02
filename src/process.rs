use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};
use chrono::Utc;
use foundationdb::{options::MutationType, FdbResult, TransactOption};
use futures::FutureExt;
use serde_mappable_seq::Key;
use twilight_model::gateway::payload::PresenceUpdate;

use crate::{helpers::bytes_to_date, keys::*, Runner};

impl Runner {
    #[inline]
    pub(crate) async fn process(&self, p: PresenceUpdate) -> Result<()> {
        self.fdb
            .transact_boxed(p, |tx, p| exec(tx, p).boxed(), TransactOption::default())
            .await?;

        Ok(())
    }
}

#[inline]
async fn exec(t: &foundationdb::Transaction, pres: &PresenceUpdate) -> FdbResult<()> {
    let time_now = Utc::now();
    let mut now = [0; 8];
    LittleEndian::write_u64(&mut now, time_now.timestamp() as u64);

    let usr = pres.user.key().to_string();
    let usr = usr.as_bytes();
    let mut game = pres.game.as_ref().map(|a| a.name.as_bytes()).unwrap_or(b"");

    if game == b"Custom Status" {
        if let Some(cus) = pres.game.as_ref().unwrap().state.as_ref() {
            game = cus.as_bytes()
        }
    }

    let first_key = fmt_first_seen_key(usr);
    let last_key = fmt_last_updated_key(usr);
    let cur_key = fmt_current_game_key(usr);

    let first = t.get(&first_key, false).await?;
    if first.is_none() {
        t.set(&first_key, &now);
    }

    let cur = t.get(&cur_key, false).await?;
    if cur.is_none() {
        t.set(&cur_key, game);
        t.set(&last_key, &now);
        return Ok(());
    }

    let cur = &*cur.unwrap();
    if cur == game {
        return Ok(());
    }

    let last_changed = t
        .get(&last_key, false)
        .await?
        .as_ref()
        .map(|v| bytes_to_date(&*v))
        .unwrap_or(time_now);

    t.set(&last_key, &now);
    t.set(&cur_key, game);

    if cur.is_empty() {
        return Ok(());
    }

    let mut to_add = [0; 8];
    LittleEndian::write_u64(
        &mut to_add,
        time_now.signed_duration_since(last_changed).num_seconds() as u64,
    );

    t.atomic_op(&fmt_user_game(usr, cur), &to_add, MutationType::Add);

    Ok(())
}
