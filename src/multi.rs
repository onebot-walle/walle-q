use std::{collections::HashMap, sync::Arc};

use cached::{SizedCache, TimedCache};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use ricq::{handler::QEvent, RQError};
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
    ActionHandler, EventHandler, GetSelfs, GetStatus, GetVersion, OneBot,
};

use crate::{
    config::QQConfig,
    database::WQDatabase,
    error::{self, map_action_parse_error},
    handler::Handler,
    login::{action_login, after_login, login_resp_to_resp, wait_qrcode},
    model::{is_wq_meta, WQMetaAction},
    WALLE_Q,
};

pub struct MultiAH {
    pub ahs: Arc<DashMap<String, (Handler, Vec<JoinHandle<()>>)>>,
    pub(crate) super_token: Option<String>,
    pub(crate) data_path: Arc<String>,
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
        data_path: Arc<String>,
    ) -> Self {
        Self {
            super_token,
            data_path,
            event_cache: Arc::new(Mutex::new(SizedCache::with_size(event_cache_size))),
            file_cache: Arc::new(Mutex::new(TimedCache::with_lifespan(60))),
            database,
            ahs: Arc::new(DashMap::default()),
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
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        for (id, cs) in config {
            let single_handler = Handler {
                client: OnceCell::default(),
                data_path: self.data_path.clone(),
                event_cache: self.event_cache.clone(),
                database: self.database.clone(),
                uploading_fragment: self.file_cache.clone(),
                infos: Arc::default(),
            };
            match single_handler
                .start(ob, (id, cs.password, cs.protocol.unwrap_or_default()))
                .await
            {
                Ok(tasks) => {
                    self.ahs.insert(
                        single_handler.get_client().unwrap().uin().await.to_string(),
                        (single_handler, tasks),
                    );
                }
                Err(e) => warn!(target: WALLE_Q, "{}", e),
            }
        }
        Ok(vec![])
    }
    async fn call<AH, EH>(&self, action: Action, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<Resp>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        let bot = action.get_self();
        if is_wq_meta(&action.action) {
            match WQMetaAction::try_from(action) {
                Ok(WQMetaAction::Login(login)) => {
                    let ah = Handler {
                        client: OnceCell::default(),
                        data_path: self.data_path.clone(),
                        event_cache: self.event_cache.clone(),
                        database: self.database.clone(),
                        uploading_fragment: self.file_cache.clone(),
                        infos: Arc::default(),
                    };
                    let (net, rx) = ah.init_client(login.bot_id.clone(), login.protocol).await;
                    let cli = ah.get_client().unwrap().clone();
                    let r =
                        action_login(&cli, &login.bot_id, login.password, &self.data_path).await;
                    match r {
                        Ok(r) => {
                            if let (r, Some(sig)) = r {
                                let base_path = self.data_path.clone();
                                let ahs = self.ahs.clone();
                                let ob = ob.clone();
                                tokio::spawn(async move {
                                    wait_qrcode(&cli, 60, &sig).await.ok();
                                    after_login(&cli, &base_path).await.ok();
                                    ah.update_infos().await.ok(); //todo
                                    if let Ok(tasks) = ah.spawn(net, rx, &ob).await {
                                        ahs.insert(cli.uin().await.to_string(), (ah, tasks));
                                    }
                                });
                                Ok(r)
                            } else {
                                if let Err(e) = after_login(&cli, &self.data_path).await {
                                    return Ok(rqe2resp(e));
                                }
                                ah.update_infos().await.ok(); //todo
                                let tasks = ah.spawn(net, rx, ob).await?;
                                self.ahs.insert(cli.uin().await.to_string(), (ah, tasks));
                                Ok(r.0)
                            }
                        }
                        Err(e) => {
                            self.unadded_client
                                .insert(login.bot_id.clone(), (ah, rx, net));
                            Ok(rqe2resp(e))
                        }
                    }
                }
                Ok(WQMetaAction::SubmitLogin(ticket)) => {
                    if let Some((_, (handler, rx, net))) =
                        self.unadded_client.remove(&ticket.bot_id)
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
                            if let Err(e) = after_login(&cli, &self.data_path).await {
                                return Ok(rqe2resp(e));
                            }
                            let tasks = handler.spawn(net, rx, ob).await?;
                            handler.update_infos().await.ok(); //todo
                            self.ahs
                                .insert(cli.uin().await.to_string(), (handler, tasks));
                        }
                        match login_resp_to_resp(&cli, resp, &self.data_path).await {
                            Ok(resp) => Ok(resp),
                            Err(e) => Ok(crate::error::rq_error(e).into()),
                        }
                    } else {
                        Ok(error::client_not_initialized("please call login first").into())
                    }
                }
                Ok(WQMetaAction::Shutdown(shutdown)) => {
                    if let Some(ref token) = self.super_token {
                        if token == shutdown.super_token.as_str() {
                            let ob = ob.clone();
                            tokio::spawn(async move { ob.shutdown(true).await });
                            Ok(Resp::ok((), ""))
                        } else {
                            Ok(error::bad_param("super_token not match").into())
                        }
                    } else {
                        Ok(error::bad_param("super_token not set").into())
                    }
                }
                Ok(WQMetaAction::Logout(token)) => {
                    if let Some(ref super_token) = self.super_token {
                        if super_token == token.super_token.as_str() {
                            if let Ok(Some(_)) = self.remove_handler(&token.bot_id, ob).await {
                                Ok(Resp::ok((), ""))
                            } else {
                                Ok(resp_error::internal_handler("bot not found").into())
                            }
                        } else {
                            Ok(error::bad_param("super_token not match").into())
                        }
                    } else {
                        Ok(error::bad_param("super_token not set").into())
                    }
                }
                Err(e) => Ok(map_action_parse_error(e).into()),
            }
        } else {
            if let Some(ah) = self.ahs.get(&bot.user_id) {
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

fn rqe2resp(e: RQError) -> Resp {
    error::rq_error(e).into()
}

impl MultiAH {
    // pub async fn add_handler<AH, EH>(
    //     &self,
    //     uin: String,
    //     password: Option<String>,
    //     protcol: u8,
    //     ob: &Arc<OneBot<AH, EH>>,
    // ) -> WalleResult<()>
    // where
    //     AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
    //     EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    // {
    //     let handler = Handler {
    //         client: OnceCell::default(),
    //         event_cache: self.event_cache.clone(),
    //         database: self.database.clone(),
    //         uploading_fragment: self.file_cache.clone(),
    //         infos: Arc::default(),
    //     };
    //     let tasks = handler.start(ob, (uin, password, protcol)).await?;
    //     self.ahs.insert(
    //         handler.get_client().unwrap().uin().await.to_string(),
    //         (handler, tasks),
    //     );
    //     Ok(())
    // }
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
            ob.handle_event(crate::parse::util::new_event(
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
            ))
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
