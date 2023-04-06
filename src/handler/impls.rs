use super::Handler;
use crate::WALLE_Q;
use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
use cached::Cached;
use colored::*;
use ricq::{
    client::{Client, Connector, DefaultConnector},
    handler::QEvent,
};
use tracing::{info, warn};
use walle_core::{
    action::Action,
    alt::ColoredAlt,
    error::{WalleError, WalleResult},
    event::Event,
    resp::Resp,
    structs::Selft,
    structs::Version,
    ActionHandler, EventHandler, GetSelfs, GetStatus, GetVersion, OneBot,
};

#[async_trait]
impl GetSelfs for Handler {
    async fn get_selfs(&self) -> Vec<Selft> {
        if let Some(true) = self
            .client
            .get()
            .map(|cli| cli.online.load(std::sync::atomic::Ordering::SeqCst))
        {
            vec![Selft {
                user_id: self.client.get().unwrap().uin().await.to_string(),
                platform: crate::PLATFORM.to_owned(),
            }]
        } else {
            vec![]
        }
    }
    async fn get_impl(&self, _: &Selft) -> String {
        crate::WALLE_Q.to_owned()
    }
}
impl GetStatus for Handler {
    fn is_good<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = bool> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        Box::pin(async move {
            self.client.get().map_or(false, |cli| {
                cli.online.load(std::sync::atomic::Ordering::SeqCst)
            })
        })
    }
}

impl GetVersion for Handler {
    fn get_version(&self) -> Version {
        Version {
            implt: crate::WALLE_Q.to_owned(),
            version: crate::VERSION.to_owned(),
            onebot_version: 12.to_string(),
        }
    }
}

impl Handler {
    /// start net connect without login
    pub async fn init_client(
        &self,
        uin: String,
        protocol: u8,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::sync::mpsc::UnboundedReceiver<QEvent>,
    ) {
        let (qevent_tx, qevent_rx) = tokio::sync::mpsc::unbounded_channel();
        let qclient = Arc::new(Client::new_with_config(
            crate::config::load_device(&uin, protocol).unwrap(),
            qevent_tx,
        ));
        let stream = DefaultConnector.connect(&qclient).await.unwrap();
        let _qcli = qclient.clone();
        let net = tokio::spawn(async move { _qcli.start(stream).await });
        self.client.set(qclient.clone()).ok();
        tokio::task::yield_now().await;
        (net, qevent_rx)
    }

    pub async fn spawn<AH, EH>(
        &self,
        net: tokio::task::JoinHandle<()>,
        mut qevent_rx: tokio::sync::mpsc::UnboundedReceiver<QEvent>,
        ob: &Arc<OneBot<AH, EH>>,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        let database = self.database.clone();
        let infos = self.infos.clone();
        let self_id = self
            .get_client()
            .map_err(WalleError::RespError)?
            .uin()
            .await;
        let event_cache = self.event_cache.clone();
        let ob = ob.clone();
        let qclient0 = self.get_client().map_err(WalleError::RespError)?.clone();
        Ok(vec![
            tokio::spawn(async move {
                while let Some(qevent) = qevent_rx.recv().await {
                    let Some(event) =
                        crate::parse::qevent2event(qevent, &database, &infos, self_id, &ob).await else {continue;};
                    tracing::info!(target: crate::WALLE_Q, "{}", event.colored_alt());
                    event_cache
                        .lock()
                        .await
                        .cache_set(event.id.clone(), event.clone());
                    ob.handle_event(event).await.ok();
                }
            }),
            tokio::spawn(async move {
                net.await.ok();
                crate::login::start_reconnect(&qclient0, "", None).await;
            }),
        ])
    }

    pub async fn update_infos(&self) -> WalleResult<()> {
        info!(target: WALLE_Q, "updating groups and friends infos");
        if let Err(e) = self
            .infos
            .update(self.get_client().map_err(WalleError::RespError)?)
            .await
        {
            warn!(target: WALLE_Q, "update infos failed: {}", e);
            return Err(WalleError::Other(e.to_string()));
        }
        info!(target: WALLE_Q, "update infos succeed");
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<Event, Action, Resp> for Handler {
    type Config = (String, Option<String>, u8); // (uin, password, protcol)
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        let (net, qevent_rx) = self.init_client(config.0.clone(), config.2).await;
        crate::login::login(
            self.get_client().map_err(WalleError::RespError)?,
            &config.0,
            config.1.clone(),
        )
        .await
        .map_err(|e| WalleError::Other(e.to_string()))?;
        self.update_infos().await?;
        self.spawn(net, qevent_rx, &ob).await
    }
    async fn call<AH, EH>(&self, action: Action, _: &Arc<OneBot<AH, EH>>) -> WalleResult<Resp>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        tracing::debug!(target: WALLE_Q, "{}", action.colored_alt());
        match self._handle(action).await {
            Ok(resp) => {
                tracing::debug!(
                    target: WALLE_Q,
                    "[{}] {}",
                    "Action Success".green(),
                    resp.data.colored_alt()
                );
                Ok(resp)
            }
            Err(e) => {
                tracing::info!(
                    target: WALLE_Q,
                    "[{} {}] {}",
                    "Action Failed".red(),
                    e.retcode,
                    e.message
                );
                Ok(e.into())
            }
        }
    }
    async fn shutdown(&self) {
        if let Some(cli) = self.client.get() {
            cli.stop(ricq::client::NetworkStatus::Stop);
        }
    }
}
