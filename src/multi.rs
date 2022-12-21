use std::{collections::HashMap, sync::Arc};

use cached::{SizedCache, TimedCache};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use ricq::{ext::common::after_login, handler::QEvent};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::warn;
use walle_core::{
    action::Action,
    error::WalleResult,
    event::{Event, StatusUpdate},
    resp::resp_error,
    resp::Resp,
    structs::{Selft, Version},
    util::GetSelf,
    ActionHandler, EventHandler, GetSelfs, GetStatus, GetVersion, OneBot, WalleError,
};

use crate::{
    config::QQConfig,
    database::WQDatabase,
    error::{self, map_action_parse_error},
    handler::Handler,
    login::{action_login, login_resp_to_resp},
    WALLE_Q,
};

pub struct MultiAH {
    pub super_token: Option<String>,
    pub(crate) ahs: DashMap<String, (Handler, Vec<JoinHandle<()>>)>,
    pub(crate) database: Arc<WQDatabase>,
    pub(crate) event_cache: Arc<Mutex<SizedCache<String, Event>>>,
    pub(crate) file_cache: Arc<Mutex<TimedCache<String, crate::handler::FragmentFile>>>,
    pub(crate) unadded_client: DashMap<
        String,
        (
            Handler,
            tokio::sync::mpsc::UnboundedReceiver<QEvent>,
            tokio::task::JoinHandle<()>,
        ),
    >,
}

impl MultiAH {
    pub fn new(
        super_token: Option<String>,
        event_cache_size: usize,
        database: Arc<WQDatabase>,
    ) -> Self {
        Self {
            super_token,
            event_cache: Arc::new(Mutex::new(SizedCache::with_size(event_cache_size))),
            file_cache: Arc::new(Mutex::new(TimedCache::with_lifespan(60))),
            database,
            ahs: DashMap::default(),
            unadded_client: DashMap::default(),
        }
    }
}

impl GetSelfs for MultiAH {
    fn get_selfs<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Vec<Selft>> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let mut selfs = vec![];
            for h in self.ahs.iter() {
                selfs.extend(h.value().0.get_selfs().await.into_iter());
            }
            selfs
        })
    }
    fn get_impl<'life0, 'life1, 'async_trait>(
        &'life0 self,
        _: &'life1 Selft,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = String> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { crate::WALLE_Q.to_owned() })
    }
}

impl GetStatus for MultiAH {
    fn is_good<'life0, 'async_trait>(
        &'life0 self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = bool> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { true })
    }
}

impl GetVersion for MultiAH {
    fn get_version(&self) -> Version {
        Version {
            implt: crate::WALLE_Q.to_owned(),
            version: crate::VERSION.to_owned(),
            onebot_version: 12.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl ActionHandler<Event, Action, Resp> for MultiAH {
    type Config = HashMap<String, QQConfig>;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        mut config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        if config.is_empty() {
            config.insert(
                String::default(),
                QQConfig {
                    password: None,
                    protocol: Some(0),
                },
            );
        }
        for (id, cs) in config {
            let ah = Handler {
                client: OnceCell::default(),
                event_cache: self.event_cache.clone(),
                database: self.database.clone(),
                uploading_fragment: self.file_cache.clone(),
                infos: Arc::default(),
            };
            match ah
                .start(ob, (id, cs.password, cs.protocol.unwrap_or_default()))
                .await
            {
                Ok(tasks) => {
                    self.ahs.insert(
                        ah.get_client().unwrap().uin().await.to_string(),
                        (ah, tasks),
                    );
                }
                Err(e) => warn!(target: WALLE_Q, "{}", e),
            }
        }
        if self.ahs.is_empty() {
            std::process::exit(1)
        }
        Ok(vec![])
    }
    async fn call<AH, EH>(&self, action: Action, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<Resp>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        match action.action.as_str() {
            "login_client" => match crate::model::LoginClient::try_from(action) {
                Ok(login) => {
                    let ah = Handler {
                        client: OnceCell::default(),
                        event_cache: self.event_cache.clone(),
                        database: self.database.clone(),
                        uploading_fragment: self.file_cache.clone(),
                        infos: Arc::default(),
                    };
                    let (net, rx) = ah.init_client(login.uin.clone(), login.protcol).await;
                    let cli = ah.get_client().unwrap().clone();
                    self.unadded_client.insert(login.uin.clone(), (ah, rx, net));
                    action_login(&cli, &login.uin, login.password)
                        .await
                        .map_err(|e| WalleError::Other(e.to_string()))
                }
                Err(e) => Ok(map_action_parse_error(e).into()),
            },
            "submit_ticket" => match crate::model::SubmitTicket::try_from(action) {
                Ok(ticket) => {
                    if let Some((_, (handler, rx, net))) =
                        self.unadded_client.remove(&ticket.user_id)
                    {
                        let cli = match handler.get_client() {
                            Ok(cli) => cli.clone(),
                            Err(e) => return Ok(e.into()),
                        };
                        let resp = match cli.submit_ticket(&ticket.ticket).await {
                            Ok(resp) => resp,
                            Err(e) => return Ok(crate::error::rq_error(e).into()),
                        };
                        if let ricq::LoginResponse::Success(_) = resp {
                            after_login(&cli).await;
                            let tasks = handler.spawn(net, rx, ob).await?;
                            handler.update_infos().await.ok(); //todo
                            self.ahs.insert(ticket.user_id, (handler, tasks));
                        }
                        match login_resp_to_resp(&cli, resp).await {
                            Ok(resp) => Ok(resp),
                            Err(e) => Ok(crate::error::rq_error(e).into()),
                        }
                    } else {
                        Ok(walle_core::resp::resp_error::who_am_i("").into())
                    }
                }
                Err(e) => Ok(map_action_parse_error(e).into()),
            },
            "shutdown" => match crate::model::Shutdown::try_from(action) {
                Ok(shutdown) => {
                    if let Some(ref token) = self.super_token {
                        if token == shutdown.super_token.as_str() {
                            let ob = ob.clone();
                            tokio::spawn(async move { ob.shutdown(true).await });
                            Ok(Resp::ok((), ""))
                        } else {
                            Ok(error::bad_param("super_token not match").into())
                        }
                    } else {
                        Ok(resp_error::bad_handler("super_token unset").into())
                    }
                }
                Err(e) => Ok(map_action_parse_error(e).into()),
            },
            _ => {
                let bot_id = action.get_self();
                if let Some(ah) = self.ahs.get(&bot_id.user_id) {
                    ah.0.call(action, ob).await
                } else if self.ahs.len() == 1 {
                    for ah in self.ahs.iter() {
                        return ah.0.call(action, ob).await;
                    }
                    Ok(resp_error::bad_handler("unreachable! How??").into())
                } else {
                    Ok(resp_error::bad_param("self_id required").into())
                }
            }
        }
    }
}

impl MultiAH {
    pub async fn add_handler<AH, EH>(
        &self,
        uin: String,
        password: Option<String>,
        protcol: u8,
        ob: &Arc<OneBot<AH, EH>>,
    ) -> WalleResult<()>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        let handler = Handler {
            client: OnceCell::default(),
            event_cache: self.event_cache.clone(),
            database: self.database.clone(),
            uploading_fragment: self.file_cache.clone(),
            infos: Arc::default(),
        };
        let tasks = handler.start(ob, (uin, password, protcol)).await?;
        self.ahs.insert(
            handler.get_client().unwrap().uin().await.to_string(),
            (handler, tasks),
        );
        Ok(())
    }
    pub async fn remove_handler<AH, EH>(
        &self,
        uin: &str,
        ob: &Arc<OneBot<AH, EH>>,
    ) -> WalleResult<Option<Handler>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        if let Some((_, (mut handler, tasks))) = self.ahs.remove(uin) {
            for task in tasks {
                task.abort();
            }
            ob.handle_event(
                crate::parse::util::new_event(
                    None,
                    (
                        walle_core::event::Meta,
                        StatusUpdate {
                            status: ob.get_status().await,
                        },
                        (),
                        crate::model::QQ,
                        crate::model::WalleQ,
                    ),
                )
                .await,
            )
            .await?;
            handler
                .get_client()
                .unwrap()
                .stop(ricq::client::NetworkStatus::Stop);
            handler.client = OnceCell::default();
            Ok(Some(handler))
        } else {
            Ok(None)
        }
    }
}
