#![feature(if_let_guard)]

pub mod config;
pub mod database;
mod handler;
mod login;
pub mod multi;
mod util;

pub(crate) mod error;
pub(crate) mod model;
pub(crate) mod parse;

pub const WALLE_Q: &str = "walle-q";
pub const PLATFORM: &str = "qq";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const LOG_PATH: &str = "./log";
pub const DATA_PATH: &str = "./data";
const IMAGE_DIR: &str = "image";
const VOICE_DIR: &str = "voice";
const FILE_DIR: &str = "file";
const CLIENT_DIR: &str = "client";
const CACHE_DIR: &str = "cache";

pub async fn init(data_path: Option<String>, log_path: Option<String>) {
    let data_path = data_path.unwrap_or(DATA_PATH.to_owned());
    macro_rules! path {
        ($dir: expr) => {
            &format!("{}/{}", data_path, $dir)
        };
    }
    tokio::fs::create_dir_all(path!(IMAGE_DIR)).await.ok();
    tokio::fs::create_dir_all(path!(FILE_DIR)).await.ok();
    tokio::fs::create_dir_all(path!(VOICE_DIR)).await.ok();
    tokio::fs::create_dir_all(path!(CLIENT_DIR)).await.ok();
    tokio::fs::remove_dir_all(path!(CACHE_DIR)).await.ok();
    tokio::fs::create_dir_all(path!(CACHE_DIR)).await.ok();
    tokio::fs::create_dir(log_path.unwrap_or(LOG_PATH.to_owned()))
        .await
        .ok();
}
