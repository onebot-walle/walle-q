use async_trait::async_trait;
use walle_core::{Action, ActionHandler, Resps};

pub(crate) struct AHandler;

#[async_trait]
impl ActionHandler<Action, Resps> for AHandler {
    async fn handle(&self, _action: Action) -> Resps {
        Resps::unsupported_action()
    }
}

use rs_qq::client::handler::{Handler, Msgs};
use walle_core::{impls::OneBot, Event};
use std::sync::Arc;
use crate::parse::Parse;

pub(crate) struct QHandler(pub(crate) Arc<OneBot>);

#[async_trait]
impl Handler for QHandler {
    async fn handle(&self, msg: Msgs) -> Result<(), Box<dyn std::error::Error>> {
        let e: Event = msg.parse();
        self.0.send_event(e)?;
        Ok(())
    }
}
