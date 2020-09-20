use played_rs::Runner;

use foundationdb::api::FdbApiBuilder;
use foundationdb::Database;
use std::env;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let network_builder = FdbApiBuilder::default()
        .build()
        .expect("fdb api initialized");
    let (runner, cond) = network_builder.build().expect("fdb network runners");

    let net_thread = std::thread::spawn(move || {
        unsafe { runner.run() }.expect("failed to run");
    });

    // Wait for the foundationDB network thread to start
    let fdb_network = cond.wait();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db: Database = foundationdb::Database::default().expect("open fdb");
    let srv = Runner::new(db, token);

    ctrlc::set_handler(move || {
        srv.close();
    })
    .expect("set interrupt handler");
    srv.start()
        .await
        .map_err(|e| println!("error running: {}", e))
        .ok();

    fdb_network.stop().expect("stop network");
    net_thread.join().expect("join fdb thread");
    Ok(())
}
