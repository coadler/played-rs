use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, TimeZone, Utc};
use foundationdb::tuple::TupleUnpack;
use foundationdb::*;
use futures::FutureExt;
use std::convert::TryInto;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::stream::StreamExt;

use anyhow::Result;
use serde_mappable_seq::Key;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_model::gateway::{payload::PresenceUpdate, Intents};

struct Server {
    fdb: Database,
    count: AtomicU64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let net = std::thread::spawn(|| {
        foundationdb::boot(|| {
            std::thread::park();
        });
    });

    tokio::time::delay_for(Duration::from_millis(500)).await;

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let db: Database = foundationdb::Database::default().expect("open fdb");

    let s: &'static Server = Box::leak(Box::new(Server {
        fdb: db,
        count: AtomicU64::new(0),
    }));

    let scheme = ShardScheme::Auto;

    let cluster = Cluster::builder(token)
        .shard_scheme(scheme)
        .intents(Intents::GUILD_PRESENCES)
        .build()
        .await?;

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let cluster_end = cluster.clone();
    ctrlc::set_handler(move || {
        println!("closing");
        cluster_end.down();
    })
    .expect("set close handler");

    tokio::spawn(async move {
        tick(&*s).await;
    });

    let mut events = cluster.events();

    while let Some((shard, event)) = events.next().await {
        let s = &*s;
        match event {
            Event::PresenceUpdate(p) => {
                tokio::spawn(async move {
                    s.count.fetch_add(1, Ordering::Relaxed);
                    s.process(*p).await.map_err(|e| println!("fuck {}", e)).ok();
                });
            }
            Event::ShardIdentifying(_) => {
                println!("shard {} identifying", shard);
            }
            Event::ShardConnecting(_) => {
                println!("shard {} connecting", shard);
            }
            Event::ShardConnected(_) => {
                println!("shard {} connected", shard);
            }
            Event::ShardDisconnected(_) => {
                println!("shard {} disconnected", shard);
            }
            Event::ShardResuming(_) => {
                println!("shard {} resuming", shard);
            }
            Event::Resumed => {
                println!("shard {} resumed", shard);
            }
            Event::Ready(r) => {
                println!("shard {} ready, {} guilds", shard, r.guilds.len());
            }
            _ => {}
        }
    }

    net.thread().unpark();
    net.join().ok();
    Ok(())
}

impl Server {
    async fn process(&self, p: PresenceUpdate) -> Result<()> {
        self.fdb
            .transact_boxed(p, |tx, p| exec(tx, p).boxed(), TransactOption::default())
            .await?;

        Ok(())
    }
}

async fn exec(t: &foundationdb::Transaction, pres: &PresenceUpdate) -> FdbResult<()> {
    let time_now = Utc::now();
    let mut now = [0; 8];
    LittleEndian::write_u64(&mut now, time_now.timestamp() as u64);

    let usr = pres.user.key().to_string();
    let usr = usr.as_bytes();
    let game = pres.game.as_ref().map(|a| a.name.as_bytes()).unwrap_or(b"");

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
        .map(|v| {
            let secs = LittleEndian::read_u64(&*v);
            Utc.timestamp(secs.try_into().unwrap(), 0)
        })
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

    t.atomic_op(
        &fmt_user_game(usr, cur),
        &to_add,
        options::MutationType::Add,
    );

    Ok(())
}

pub struct Response {
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub games: Vec<Entry>,
}

pub struct Entry {
    pub name: String,
    pub dur: u32,
}

async fn read_exec<T: AsRef<[u8]>>(t: &foundationdb::Transaction, user: T) -> FdbResult<()> {
    let vals: Vec<Entry> = t
        .get_range(&fmt_user_range(user.as_ref()), 1, true)
        .await?
        .into_iter()
        .map(|v| {
            // asdf
            let (_, _, user, game): (Vec<u8>, u16, String, String) =
                tuple::unpack(v.key()).unwrap();
            Entry {
                name: "asdf".to_string(),
                dur: 0,
            }
        })
        .collect();

    Ok(())
}

async fn tick(p: &Server) -> ! {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;

        let c = p.count.swap(0, Ordering::Relaxed);
        println!("{} events", c);
    }
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
