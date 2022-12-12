use std::sync::Arc;
use std::{collections::HashMap, io::Read};

use chrono::{Offset, TimeZone};
use rand::SeedableRng;
use ricq::version::get_version;
use ricq::Config as RsQQConfig;
use ricq::{device::Device, version::Protocol};
use serde::{Deserialize, Serialize};
use tracing::metadata::LevelFilter;
use tracing::{info, warn};
use walle_core::config::ImplConfig;

use crate::database::WQDatabase;
use crate::WALLE_Q;

type IOResult<T> = Result<T, std::io::Error>;

const CONFIG_PATH: &str = "walle-q.toml";
const DEVICE_PATH: &str = "device.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub qq: HashMap<String, QQConfig>,
    pub meta: MetaConfig,
    pub onebot: ImplConfig,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct QQConfig {
    pub(crate) password: Option<String>,
    pub(crate) protocol: Option<u8>,
}

trait NewConfig: Sized {
    fn new_config() -> Self;
    fn ser(&self) -> IOResult<String>;
    fn de(s: &str) -> IOResult<Self>;
}

trait LoadConfig: for<'de> Deserialize<'de> + Serialize + NewConfig {
    fn save_to_file(&self, path: &str) -> IOResult<()> {
        let data = self.ser()?;
        std::fs::write(path, data)
    }

    fn load_from_file(path: &str) -> IOResult<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        Self::de(&data)
    }

    fn load_or_new(path: &str) -> IOResult<Self> {
        info!(target: WALLE_Q, "loading {}", path);
        match Self::load_from_file(path) {
            Ok(config) => {
                info!(target: WALLE_Q, "success load from {}", path);
                Ok(config)
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Other => {
                    warn!("Serialize error: {}", e);
                    Err(e)
                }
                _ => {
                    warn!("open {} failed: {}", path, e);
                    info!(target: WALLE_Q, "creating new {}", path);
                    let config = Self::new_config();
                    config.save_to_file(path)?;
                    Ok(config)
                }
            },
        }
    }
}

impl NewConfig for Config {
    fn new_config() -> Self {
        Self::default()
    }

    fn ser(&self) -> IOResult<String> {
        toml::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn de(s: &str) -> IOResult<Self> {
        toml::from_str(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl LoadConfig for Config {}

impl NewConfig for Device {
    fn new_config() -> Self {
        Self::random()
    }

    fn ser(&self) -> IOResult<String> {
        serde_json::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn de(s: &str) -> IOResult<Self> {
        serde_json::from_str(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl LoadConfig for Device {}

impl Config {
    pub fn load() -> Result<Self, std::io::Error> {
        Self::load_or_new(CONFIG_PATH)
    }
}

pub(crate) fn load_device(uin: &str, protocol: u8) -> IOResult<RsQQConfig> {
    Ok(RsQQConfig {
        device: Device::load_or_new(&format!(
            "{}/{}-{}-{}",
            crate::CLIENT_DIR,
            uin,
            protocol,
            DEVICE_PATH
        ))
        .unwrap_or_else(|_| {
            Device::random_with_rng(&mut rand::prelude::StdRng::seed_from_u64(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ))
        }),
        version: get_version(Protocol::try_from(protocol).unwrap_or_default()),
    })
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaConfig {
    pub log_level: LogLevel,
    pub event_cache_size: usize,
    pub sled: bool,
    pub leveldb: bool,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::default(),
            event_cache_size: 10,
            sled: true,
            leveldb: false,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(log: LogLevel) -> Self {
        match log {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
        }
    }
}

impl MetaConfig {
    pub fn subscribe(&self) {
        let offset = chrono::Local
            .timestamp_opt(0, 0)
            .unwrap()
            .offset()
            .fix()
            .local_minus_utc();
        let timer = tracing_subscriber::fmt::time::OffsetTime::new(
            time::UtcOffset::from_whole_seconds(offset).unwrap(),
            time::macros::format_description!(
                "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]"
            ),
        );
        let filter = tracing_subscriber::filter::Targets::new()
            .with_default(LevelFilter::INFO)
            .with_targets([
                (crate::WALLE_Q, self.log_level),
                (walle_core::WALLE_CORE, self.log_level),
                (walle_core::obc::OBC, self.log_level),
            ]);
        let file_appender =
            tracing_appender::rolling::daily(crate::LOG_PATH, format!("{}.log", crate::WALLE_Q));
        use tracing_subscriber::{
            prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
        };
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_timer(timer.clone())
                    .with_filter(filter),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_appender)
                    .with_timer(timer)
                    .with_ansi(false)
                    .with_filter(
                        tracing_subscriber::filter::Targets::new().with_default(LevelFilter::WARN),
                    ),
            )
            .init();
    }

    pub fn db(&self) -> Arc<WQDatabase> {
        let mut db = WQDatabase::default();
        if self.sled {
            db = db.sled();
        }
        if self.leveldb {
            db = db.level()
        }
        Arc::new(db)
    }
}
