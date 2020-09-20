use played_rs::Runner;

use foundationdb::api::FdbApiBuilder;
use foundationdb::Database;

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

    let db: Database = foundationdb::Database::default().expect("open fdb");
    let srv = Runner::new(db, "");

    let res = srv.read("105484726235607040").await;
    dbg!(&res);

    fdb_network.stop().expect("stop network");
    net_thread.join().expect("join fdb thread");
    Ok(())
}
