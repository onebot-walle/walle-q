#![feature(if_let_guard)]

use std::sync::Arc;

use clap::Parser;
use walle_core::obc::ImplOBC;

mod command;
mod config;
mod database;
mod handler;
mod login;
mod multi;
mod util;

pub(crate) mod error;
pub(crate) mod model;
pub(crate) mod parse;

const WALLE_Q: &str = "Walle-Q";
const PLATFORM: &str = "qq";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const LOG_PATH: &str = "./log";
const IMAGE_CACHE_DIR: &str = "./data/image";
const VOICE_CACHE_DIR: &str = "./data/voice";
const FILE_CACHE_DIR: &str = "./data/file";
const CLIENT_DIR: &str = "./data/client";

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

async fn init() {
    tokio::fs::create_dir_all(IMAGE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(FILE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(VOICE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(CLIENT_DIR).await.ok();
    tokio::fs::create_dir(crate::LOG_PATH).await.ok();
}
