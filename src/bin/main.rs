#![feature(if_let_guard)]

use std::sync::Arc;

use clap::Parser;
use walle_core::obc::ImplOBC;

use walle_q::command;
use walle_q::config;
use walle_q::multi;

use walle_q::init;

const WALLE_Q: &str = "Walle-Q";
const PLATFORM: &str = "qq";

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
    let joins = Arc::new(walle_core::OneBot::<_, _, 12>::new(
        ah,
        ImplOBC::<walle_core::event::Event>::new(WALLE_Q.to_string(), PLATFORM.to_string()),
    ))
    .start(config.qq, config.onebot, false)
    .await
    .unwrap();
    for join in joins {
        join.await.unwrap();
    }
}
