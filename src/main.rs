use std::sync::Arc;

use rs_qq::client::Client;

mod config;
mod handler;
mod login;
mod parse;

const WALLE_Q: &str = "Walle-Q";

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("Walle-core=debug");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let config = config::Config::load_or_new();
    let ob = walle_core::impls::OneBot::new(
        WALLE_Q,
        "qq",
        &config.qq.uin.to_string(),
        config.onebot,
        Arc::new(handler::AHandler),
    )
    .arc();
    let qh = handler::QHandler(ob.clone());
    let qclient = Arc::new(Client::new_with_config(config.qq, qh).await);
    let net = qclient.run().await;
    login::login(&qclient).await.unwrap();

    ob.run().await.unwrap();
    net.await.unwrap();
}
