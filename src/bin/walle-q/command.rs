use clap::Parser;

use crate::config::{LogLevel, MetaConfig};

#[derive(Parser, Debug, Default)]
#[clap(name = "Walle-Q",
       author = "AbrahumLink",
       version = env!("CARGO_PKG_VERSION"),
       about = "Walle-Q is a Onebot implementation in Rust")]
pub struct Comm {
    #[clap(short, long, help = "set log level to error")]
    pub error: bool,

    #[clap(short, long, help = "set log level to warn")]
    pub warn: bool,

    #[clap(short, long, help = "set log level to info")]
    pub info: bool,

    #[clap(short, long, help = "set log level to debug")]
    pub debug: bool,

    #[clap(short, long, help = "set log level to trace")]
    pub trace: bool,

    #[clap(long, help = "this size of event cache will be used. (Default: 10)")]
    pub event_cache_size: Option<usize>,
}

impl Comm {
    pub fn merge(self, meta: &mut MetaConfig) {
        fn merge_option<T>(a: &mut T, b: Option<T>) {
            if let Some(b) = b {
                *a = b;
            }
        }
        merge_option(&mut meta.event_cache_size, self.event_cache_size);
        if self.warn {
            meta.log_level = LogLevel::Warn;
        } else if self.error {
            meta.log_level = LogLevel::Error;
        } else if self.info {
            meta.log_level = LogLevel::Info;
        } else if self.debug {
            meta.log_level = LogLevel::Debug;
        } else if self.trace {
            meta.log_level = LogLevel::Trace;
        }
    }
}
