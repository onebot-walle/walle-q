use std::{collections::HashMap, sync::Arc};

use cached::{SizedCache, TimedCache};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::warn;
use walle_core::{
    action::Action, error::WalleResult, event::Event, resp::resp_error, resp::Resp, structs::Selft,
    util::GetSelf, ActionHandler, EventHandler, GetSelfs, GetStatus, OneBot,
};

use crate::{config::QQConfig, database::WQDatabase, handler::Handler, WALLE_Q};

pub struct MultiAH {
    pub(crate) ahs: DashMap<String, (Handler, Vec<JoinHandle<()>>)>,
    pub(crate) database: Arc<WQDatabase>,
    pub(crate) event_cache: Arc<Mutex<SizedCache<String, Event>>>,
    pub(crate) file_cache: Arc<Mutex<TimedCache<String, crate::handler::FragmentFile>>>,
}

impl MultiAH {
    pub fn new(event_cache_size: usize, database: Arc<WQDatabase>) -> Self {
        Self {
            event_cache: Arc::new(Mutex::new(SizedCache::with_size(event_cache_size))),
            file_cache: Arc::new(Mutex::new(TimedCache::with_lifespan(60))),
            database,
            ahs: DashMap::default(),
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
