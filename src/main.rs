use std::sync::Arc;

use cached::Cached;
use rs_qq::client::Client;

mod command;
mod config;
mod database;
mod handler;
mod login;
pub(crate) mod parse;

const WALLE_Q: &str = "Walle-Q";
const VERSION: &str = env!("CARGO_PKG_VERSION");

use clap::Parser;
use database::DatabaseInit;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SLED_DB: database::sled::SledDb = database::sled::SledDb::init();
}

#[tokio::main]
async fn main() {
    let mut comm = command::Comm::parse();
    let config = config::Config::load().unwrap();
    comm.merge(config.command);
    comm.subscribe();

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
    tokio::spawn(async move { qcli2.start(stream).await });
    tokio::task::yield_now().await;
    login::login(&qclient, &config.qq).await.unwrap();

    let cache = Arc::new(tokio::sync::Mutex::new(cached::SizedCache::with_size(100)));
    let ob = walle_core::impls::OneBot::new(
        WALLE_Q,
        "qq",
        &self_id.to_string(),
        config.onebot.clone(),
        Arc::new(handler::Handler(qclient.clone(), cache.clone())),
    )
    .arc();
    if !comm.v11 {
        ob.run().await.unwrap();
        while let Some(msg) = rx.recv().await {
            if let Some(event) = crate::parse::qevent2event(&ob, msg).await {
                tracing::info!("{:?}", event);
                cache
                    .lock()
                    .await
                    .cache_set(event.id.clone(), event.clone());
                ob.send_event(event).unwrap();
            }
        }
    } else {
        tracing::warn!(target: WALLE_Q, "Using Onebot v11 standard");
        let ob11 = walle_v11::impls::OneBot11::new(
            WALLE_Q,
            "qq",
            &self_id.to_string(),
            config.onebot,
            Arc::new(handler::v11::V11Handler(ob.clone())),
        )
        .arc();
        ob11.run().await.unwrap();
        while let Some(msg) = rx.recv().await {
            parse::v11::meta_event_process(&ob11, &msg).await;
            if let Some(event) = crate::parse::qevent2event(&ob, msg).await {
                cache
                    .lock()
                    .await
                    .cache_set(event.id.clone(), event.clone());
                let e = event.try_into().unwrap();
                tracing::info!("{:?}", e);
                ob11.send_event(e).unwrap();
            }
        }
    }
}
