use clap::Parser;
use std::sync::Arc;
use walle_core::obc::ImplOBC;
use walle_q::{command, config, init, multi, WALLE_Q};

#[tokio::main]
async fn main() {
    let mut comm = command::Comm::parse();
    let config = match config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            println!("load config failed: {e}");
            std::process::exit(1)
        }
    };
    comm.merge(config.meta);
    comm.subscribe();
    let wqdb = comm.db();
    init().await;

    let ah = multi::MultiAH::new(comm.event_cache_size.unwrap_or(100), wqdb.clone());
    let joins = Arc::new(walle_core::OneBot::new(
        ah,
        ImplOBC::new(WALLE_Q.to_owned()),
    ))
    .start(config.qq, config.onebot, false)
    .await
    .unwrap();
    for join in joins {
        join.await.unwrap();
    }
}
