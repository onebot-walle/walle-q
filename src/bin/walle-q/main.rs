use clap::Parser;
use std::sync::Arc;
use walle_core::obc::ImplOBC;
use walle_q::{config, init, multi, WALLE_Q};

mod command;

#[tokio::main]
async fn main() {
    let comm = command::Comm::parse();
    let mut config = match config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            println!("load config failed: {e}");
            std::process::exit(1)
        }
    };
    comm.merge(&mut config.meta);
    config.meta.subscribe();
    init().await;

    let ah = multi::MultiAH::new(config.meta.event_cache_size, config.meta.db());
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
