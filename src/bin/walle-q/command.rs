use clap::Parser;

use crate::config::{LogLevel, MetaConfig};

#[derive(Parser, Debug, Default)]
#[clap(name = "Walle-Q",
       author = "AbrahumLink",
       version = env!("CARGO_PKG_VERSION"),
       about = "Walle-Q is a Onebot implementation in Rust")]
pub struct Comm {
    #[clap(long, help = "set log level")]
    pub log_level: Option<LogLevel>,

    #[clap(long, help = "this size of event cache will be used. (Default: 10)")]
    pub event_cache_size: Option<usize>,

    #[clap(long, help = "toml file path.(Default: walle-q.toml)")]
    pub toml_path: Option<String>,

    #[clap(long, help = "json config")]
    pub json_config: Option<String>,
}

impl Comm {
    pub fn merge(self, meta: &mut MetaConfig) {
        fn merge_option<T>(a: &mut T, b: Option<T>) {
            if let Some(b) = b {
                *a = b;
            }
        }
        merge_option(&mut meta.event_cache_size, self.event_cache_size);
        merge_option(&mut meta.log_level, self.log_level);
    }
    pub fn config(self) -> walle_q::config::Config {
        let mut config = match walle_q::config::Config::load_from_toml_file(None) {
            Ok(config) => config,
            Err(e) => {
                println!("load config failed: {e}");
                std::process::exit(1)
            }
        };
        self.merge(&mut config.meta);
        config
    }
}
