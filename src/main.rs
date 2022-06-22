#![feature(if_let_guard)]

use std::sync::Arc;

use clap::Parser;
use extra::WQEvent;
use tokio::sync::Mutex;

mod command;
mod config;
mod database;
pub(crate) mod error;
pub(crate) mod extra;
mod handler;
mod login;
pub(crate) mod parse;

const WALLE_Q: &str = "Walle-Q";
const PLATFORM: &str = "qq";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOG_PATH: &str = "./log";
const IMAGE_CACHE_DIR: &str = "./data/image";
const VOICE_CACHE_DIR: &str = "./data/voice";
const FILE_CACHE_DIR: &str = "./data/file";

type WQResp = walle_core::Resps<extra::WQEvent>;

#[tokio::main]
async fn main() {
    let mut comm = command::Comm::parse();
    let config = config::Config::load().unwrap();
    comm.merge(config.command);
    comm.subscribe();
    let wqdb = comm.db();
    init().await;

    let event_cache = Arc::new(Mutex::new(cached::SizedCache::with_size(
        comm.event_cache_size.unwrap_or(100),
    )));
    let uploading_fragment = Mutex::new(cached::TimedCache::with_lifespan(60));
    let ah = handler::Handler {
        client: once_cell::sync::OnceCell::new(),
        event_cache,
        database: wqdb.clone(),
        uploading_fragment,
    };
    let joins = Arc::new(walle_core::onebot::OneBot::<_, _, 12>::new(
        ah,
        walle_core::onebot::obc::ImplOBC::<WQEvent>::new(
            "".to_string(),
            WALLE_Q.to_string(),
            PLATFORM.to_string(),
        ),
    ))
    .start(config.qq, config.onebot, false)
    .await
    .unwrap();
    for join in joins {
        join.await.unwrap();
    }

    // 网络断开后自动重连
    // net.await.ok();
    // login::start_reconnect(&qclient, &config.qq).await;
}

async fn init() {
    tokio::fs::create_dir_all(crate::IMAGE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(crate::FILE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(crate::VOICE_CACHE_DIR).await.ok();
    tokio::fs::create_dir(crate::LOG_PATH).await.ok();
}
