use async_trait::async_trait;
use walle_core::{Action, ActionHandler, Resps};

pub(crate) struct AHandler;

#[async_trait]
impl ActionHandler<Action, Resps> for AHandler {
    async fn handle(&self, _action: Action) -> Resps {
        Resps::unsupported_action()
    }
}

use crate::parse::Parse;
use rs_qq::client::handler::{Handler, Msg};
use std::sync::Arc;
use walle_core::{impls::OneBot, Event};

pub(crate) struct QHandler(pub(crate) Arc<OneBot>);

#[async_trait]
impl Handler for QHandler {
    async fn handle(&self, msg: Msg) {
        let e: Event = msg.parse();
        self.0.send_event(e).unwrap();
    }
}
