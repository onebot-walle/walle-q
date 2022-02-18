use clap::{ArgEnum, Parser};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

#[derive(Parser, Serialize, Deserialize, Debug, Default)]
#[clap(name = "Walle-Q",
       author = "AbrahumLink",
       version = env!("CARGO_PKG_VERSION"),
       about = "Walle-Q is a Onebot implementation in Rust")]
pub(crate) struct Comm {
    #[clap(long, arg_enum, help = "set global log level")]
    pub log: Option<LogLevel>,

    #[clap(long, help = "use Onebot v11 standard instead of v12 (todo)")]
    #[serde(default)]
    pub v11: bool,
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
        let timer =
            tracing_subscriber::fmt::time::LocalTime::new(time::macros::format_description!(
                "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]"
            ));
        let filter = tracing_subscriber::filter::Targets::new()
            .with_default(self.log.clone().unwrap_or_default())
            .with_target("sled", LevelFilter::WARN);
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_timer(timer))
            .with(filter)
            .init();
        // tracing_subscriber::fmt()
        //     .with_timer(timer)
        //     .(filter)
        //     .init();
    }

    pub(crate) fn merge(&mut self, other: Self) {
        if let Some(log) = other.log {
            self.log = Some(log);
        }
        if other.v11 && !self.v11 {
            self.v11 = true;
        }
    }
}
