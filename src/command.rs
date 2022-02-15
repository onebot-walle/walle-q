use clap::{ArgEnum, Parser};
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize, Debug, Default)]
#[clap(name = "Walle-Q",
       author = "AbrahumLink",
       version = env!("CARGO_PKG_VERSION"),
       about = "Walle-Q is a Onebot implementation in Rust")]
pub(crate) struct Comm {
    #[clap(long, arg_enum, help = "set global log level")]
    pub log: Option<LogLevel>,

    #[clap(long, help = "use Onebot v11 standard instead of v12 (todo)")]
    pub v11: Option<bool>,
}

#[derive(ArgEnum, Clone, Serialize, Deserialize, Debug)]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
        .to_string()
    }
}

impl Comm {
    pub(crate) fn subscribe(&self) {
        let timer =
            tracing_subscriber::fmt::time::LocalTime::new(time::macros::format_description!(
                "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]"
            ));
        let log = self.log.as_ref().unwrap_or(&LogLevel::Info);
        let env = tracing_subscriber::EnvFilter::from(format!("sled=warn,{}", log.to_string()));
        tracing_subscriber::fmt()
            .with_env_filter(env)
            .with_timer(timer)
            .init();
    }

    pub(crate) fn merge(&mut self, other: Self) {
        if let Some(log) = other.log {
            self.log = Some(log);
        }
        if let Some(v11) = other.v11 {
            self.v11 = Some(v11);
        }
    }
}
