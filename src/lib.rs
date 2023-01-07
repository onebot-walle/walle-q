#![feature(if_let_guard)]

pub mod config;
mod database;
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
const IMAGE_DIR: &str = "./data/image";
const VOICE_DIR: &str = "./data/voice";
const FILE_DIR: &str = "./data/file";
const CLIENT_DIR: &str = "./data/client";
const CACHE_DIR: &str = "./data/cache";

pub async fn init() {
    tokio::fs::create_dir_all(IMAGE_DIR).await.ok();
    tokio::fs::create_dir_all(FILE_DIR).await.ok();
    tokio::fs::create_dir_all(VOICE_DIR).await.ok();
    tokio::fs::create_dir_all(CLIENT_DIR).await.ok();
    tokio::fs::remove_dir_all(CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(CACHE_DIR).await.ok();
    tokio::fs::create_dir(crate::LOG_PATH).await.ok();
}
