use std::{collections::HashMap, sync::Arc};

use cached::{SizedCache, TimedCache};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::warn;
use walle_core::{
    error::WalleResult, resp::resp_error, util::SelfId, ActionHandler, EventHandler, OneBot,
};

use crate::{
    config::QQConfig,
    database::WQDatabase,
    extra::{WQAction, WQEvent},
    handler::Handler,
    WQResp, WALLE_Q,
};

pub struct MultiAH {
    pub(crate) ahs: DashMap<String, (Handler, Vec<JoinHandle<()>>)>,
    pub(crate) database: Arc<WQDatabase>,
    pub(crate) event_cache: Arc<Mutex<SizedCache<String, WQEvent>>>,
}

impl MultiAH {
    pub fn new(event_cache_size: usize, database: Arc<WQDatabase>) -> Self {
        Self {
            event_cache: Arc::new(Mutex::new(SizedCache::with_size(event_cache_size))),
            database,
            ahs: DashMap::default(),
        }
    }
}

#[async_trait::async_trait]
impl ActionHandler<WQEvent, WQAction, WQResp, 12> for MultiAH {
    type Config = HashMap<String, QQConfig>;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, 12>>,
        mut config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
        EH: EventHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
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
                uploading_fragment: Mutex::new(TimedCache::with_lifespan(60)),
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
    async fn call<AH, EH>(&self, action: WQAction, ob: &OneBot<AH, EH, 12>) -> WalleResult<WQResp>
    where
        AH: ActionHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
        EH: EventHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
    {
        let bot_id = action.self_id();
        if let Some(ah) = self.ahs.get(&bot_id) {
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
