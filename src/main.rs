use std::sync::Arc;

use parse::Parser;
use rs_qq::client::Client;

mod config;
mod database;
mod handler;
mod login;
mod parse;

const WALLE_Q: &str = "Walle-Q";
const VERSION: &str = env!("CARGO_PKG_VERSION");

use database::DatabaseInit;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SLED_DB: database::sled::SledDb = database::sled::SledDb::init();
}

#[tokio::main]
async fn main() {
    let timer = tracing_subscriber::fmt::time::LocalTime::new(time::macros::format_description!(
        "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]"
    ));
    let env = tracing_subscriber::EnvFilter::from("rs_qq=debug,sled=warn,info");
    tracing_subscriber::fmt()
        .with_env_filter(env)
        .with_timer(timer)
        .init();
    let config = config::Config::load().unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let self_id = config.qq.uin.unwrap_or(0);
    let qclient = Arc::new(Client::new_with_config(
        config::load_device(&config.qq).unwrap(),
        tx,
    ));
    let ob = walle_core::impls::OneBot::new(
        WALLE_Q,
        "qq",
        &self_id.to_string(),
        config.onebot,
        Arc::new(handler::AHandler(qclient.clone())),
    )
    .arc();
    let qcli2 = qclient.clone();
    tokio::spawn(async move { qcli2.start().await.unwrap() });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    login::login(&qclient, &config.qq).await.unwrap();

    ob.run().await.unwrap();
    while let Some(msg) = rx.recv().await {
        if let Some(event) = ob.parse(msg).await {
            tracing::info!("{:?}", event);
            ob.send_event(event).unwrap();
        }
    }
}
