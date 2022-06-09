#![feature(if_let_guard)]

use std::sync::Arc;

use cached::Cached;
use clap::Parser;
use ricq::client::Client;
use tokio::sync::Mutex;
use walle_core::ColoredAlt;

mod command;
mod config;
mod database;
pub(crate) mod error;
pub(crate) mod extra;
mod handler;
mod login;
pub(crate) mod parse;

const WALLE_Q: &str = "Walle-Q";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOG_PATH: &str = "./log";
const IMAGE_CACHE_DIR: &str = "./data/image";
const FILE_CACHE_DIR: &str = "./data/file";

type WQResp = walle_core::Resps<extra::WQEvent>;
type OneBot =
    walle_core::impls::CustomOneBot<extra::WQEvent, extra::WQAction, WQResp, handler::Handler, 12>;

#[tokio::main]
async fn main() {
    let mut comm = command::Comm::parse();
    let config = config::Config::load().unwrap();
    comm.merge(config.command);
    comm.subscribe();
    let wqdb = comm.db();
    init().await;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let self_id = config.qq.uin.unwrap_or(0);
    let qclient = Arc::new(Client::new_with_config(
        config::load_device(&config.qq).unwrap(),
        tx,
    ));
    let qcli2 = qclient.clone();
    let stream = tokio::net::TcpStream::connect(qclient.get_address())
        .await
        .unwrap();
    let net = tokio::spawn(async move { qcli2.start(stream).await });
    tokio::task::yield_now().await;
    login::login(&qclient, &config.qq).await.unwrap();

    let cache = Arc::new(Mutex::new(cached::SizedCache::with_size(
        comm.event_cache_size.unwrap_or(100),
    )));
    let ob = OneBot::new(
        WALLE_Q,
        "qq",
        &self_id.to_string(),
        config.onebot.clone(),
        handler::Handler {
            client: qclient.clone(),
            event_cache: cache.clone(),
            database: wqdb.clone(),
            uploading_fragment: Mutex::new(cached::TimedCache::with_lifespan(60)),
        },
    )
    .arc();

    // start onebot task
    tokio::spawn(async move {
        // if !comm.v11 {
        ob.run().await.unwrap();
        while let Some(msg) = rx.recv().await {
            if let Some(event) = crate::parse::qevent2event(&ob, msg, &wqdb).await {
                if let Some(alt) = event.colored_alt() {
                    tracing::info!(target: WALLE_Q, "{}", alt);
                }
                cache
                    .lock()
                    .await
                    .cache_set(event.id.clone(), event.clone());
                ob.send_event(event).unwrap();
            }
        }
        // } else {
        //     tracing::warn!(target: WALLE_Q, "Using Onebot v11 standard");
        //     let ob11 = walle_v11::impls::OneBot11::new(
        //         WALLE_Q,
        //         "qq",
        //         &self_id.to_string(),
        //         config.onebot,
        //         handler::v11::V11Handler(ob.clone()),
        //     )
        //     .arc();
        //     ob11.run().await.unwrap();
        //     while let Some(msg) = rx.recv().await {
        //         parse::v11::meta_event_process(&ob11, &msg).await;
        //         if let Some(event) = crate::parse::qevent2event(&ob, msg, &wqdb).await {
        //             cache
        //                 .lock()
        //                 .await
        //                 .cache_set(event.id.clone(), event.clone());
        //             if let Some(alt) = event.colored_alt() {
        //                 tracing::info!(target: WALLE_Q, "{}", alt);
        //             }
        //             let e: walle_v11::Event = event.try_into().unwrap();
        //             ob11.send_event(e).unwrap();
        //         }
        //     }
        // }
    });

    // 网络断开后自动重连
    net.await.ok();
    login::start_reconnect(&qclient, &config.qq).await;
}

async fn init() {
    tokio::fs::create_dir_all(crate::IMAGE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(crate::FILE_CACHE_DIR).await.ok();
    tokio::fs::create_dir(crate::LOG_PATH).await.ok();
}
