use clap::Parser;
use std::sync::Arc;
use walle_core::obc::ImplOBC;
use walle_q::{config, init, multi, WALLE_Q};

mod command;

#[tokio::main]
async fn main() {
    let comm = command::Comm::parse();
    let config = comm.config();
    config.meta.subscribe();
    init().await;

    let ah = multi::MultiAH::new(
        config.meta.super_token.clone(),
        config.meta.event_cache_size,
        config.meta.db(),
    );
    let ob = Arc::new(walle_core::OneBot::new(
        ah,
        ImplOBC::new(WALLE_Q.to_owned()),
    ));
    ob.start(config.qq, config.onebot, false).await.unwrap();
    ob.wait_all().await;
}
