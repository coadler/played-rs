use anyhow::Result;
use foundationdb::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Duration;
use tokio::stream::StreamExt;
use tokio::sync::oneshot;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_model::gateway::Intents;

pub struct Runner {
    pub(crate) fdb: Database,
    pub(crate) token: String,
    pub(crate) count: AtomicU64,
    pub(crate) sigs: RwLock<Option<oneshot::Sender<()>>>,
}

impl Runner {
    pub fn new<T: AsRef<str>>(db: Database, token: T) -> &'static Runner {
        Box::leak(Box::new(Runner {
            fdb: db,
            token: token.as_ref().to_string(),
            count: AtomicU64::new(0),
            sigs: RwLock::new(None),
        }))
    }

    pub fn close(&self) {
        let tx = self.sigs.write().unwrap().take();
        match tx {
            Some(tx) => tx.send(()).unwrap(),
            _ => {}
        };
    }

    pub async fn start(&'static self) -> Result<()> {
        let (tx, rx) = oneshot::channel::<()>();
        *self.sigs.write().unwrap() = Some(tx);

        let scheme = ShardScheme::Auto;

        let cluster = Cluster::builder(&self.token)
            .shard_scheme(scheme)
            .intents(Intents::GUILD_PRESENCES)
            .build()
            .await?;

        let cluster_spawn = cluster.clone();
        tokio::spawn(async move {
            cluster_spawn.up().await;
        });

        let cluster_end = cluster.clone();
        tokio::spawn(async move {
            rx.await.unwrap();
            println!("closing");
            cluster_end.down();
        });

        tokio::spawn(async move {
            tick(&*self).await;
        });

        let mut events = cluster.events();
        while let Some((shard, event)) = events.next().await {
            let s = &*self;
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

        Ok(())
    }
}

async fn tick(p: &Runner) -> ! {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;

        let c = p.count.swap(0, Ordering::Relaxed);
        println!("{} events", c);
    }
}
