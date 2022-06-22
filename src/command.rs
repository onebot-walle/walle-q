use std::sync::Arc;

use clap::{ArgEnum, Parser};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    Layer,
};

use crate::WALLE_Q;

#[derive(Parser, Serialize, Deserialize, Debug, Default)]
#[clap(name = "Walle-Q",
       author = "AbrahumLink",
       version = env!("CARGO_PKG_VERSION"),
       about = "Walle-Q is a Onebot implementation in Rust")]
pub(crate) struct Comm {
    #[clap(long, arg_enum, help = "set global log level")]
    pub log: Option<LogLevel>,

    #[clap(long, help = "use Onebot v11 standard")]
    #[serde(default)]
    pub v11: bool,

    #[clap(long, help = "this size of event cache will be used. (Default: 100)")]
    pub event_cache_size: Option<usize>,

    #[clap(long, help = "time zone for log. (Default: +8)")]
    pub time_zone: Option<i8>,

    #[clap(long, help = "Enable SledDb")]
    #[serde(default)]
    pub sled: bool,

    #[clap(long, help = "Disable LevelDb")]
    #[serde(default)]
    pub disable_leveldb: bool,
}

#[derive(ArgEnum, Clone, Serialize, Deserialize, Debug)]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
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

impl Comm {
    pub(crate) fn subscribe(&self) {
        let timer = tracing_subscriber::fmt::time::OffsetTime::new(
            time::UtcOffset::from_hms(*self.time_zone.as_ref().unwrap_or(&8), 0, 0).unwrap(),
            time::macros::format_description!(
                "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]"
            ),
        );
        let log_level = self.log.clone().unwrap_or_default();
        let filter = tracing_subscriber::filter::Targets::new()
            .with_default(LevelFilter::INFO)
            .with_targets([
                (WALLE_Q, log_level.clone()),
                ("Walle-core", log_level.clone()),
                ("Walle-OBC", log_level),
            ]);
        let file_appender =
            tracing_appender::rolling::daily(crate::LOG_PATH, format!("{}.log", crate::WALLE_Q));
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

    pub(crate) fn db(&self) -> Arc<crate::database::WQDatabase> {
        let mut db = crate::database::WQDatabase::default();
        if !self.disable_leveldb {
            db = db.level();
        }
        if self.sled {
            db = db.sled();
        }
        Arc::new(db)
    }

    pub(crate) fn merge(&mut self, other: Self) {
        fn merge_option<T>(a: &mut Option<T>, b: Option<T>) {
            if let Some(b) = b {
                *a = Some(b);
            }
        }
        fn merge_bool(a: &mut bool, b: bool) {
            *a = *a || b;
        }

        merge_option(&mut self.log, other.log);
        merge_option(&mut self.event_cache_size, other.event_cache_size);
        merge_bool(&mut self.v11, other.v11);
        merge_bool(&mut self.sled, other.sled);
        merge_bool(&mut self.disable_leveldb, other.disable_leveldb);
    }
}
