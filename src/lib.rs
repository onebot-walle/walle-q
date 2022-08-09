#![feature(if_let_guard)]

pub mod command;
pub mod config;
pub mod database;
pub mod handler;
pub mod login;
pub mod multi;
pub mod util;

pub(crate) mod error;
pub(crate) mod model;
pub(crate) mod parse;

const WALLE_Q: &str = "Walle-Q";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const LOG_PATH: &str = "./log";
const IMAGE_CACHE_DIR: &str = "./data/image";
const VOICE_CACHE_DIR: &str = "./data/voice";
const FILE_CACHE_DIR: &str = "./data/file";
const CLIENT_DIR: &str = "./data/client";

pub async fn init() {
    tokio::fs::create_dir_all(IMAGE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(FILE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(VOICE_CACHE_DIR).await.ok();
    tokio::fs::create_dir_all(CLIENT_DIR).await.ok();
    tokio::fs::create_dir(crate::LOG_PATH).await.ok();
}
