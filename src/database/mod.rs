pub(crate) mod leveldb;
pub(crate) mod message;
pub(crate) mod simage;
pub(crate) mod sleddb;
pub(crate) mod voice;

pub use message::*;
pub use simage::*;
pub use voice::*;
use walle_core::{event::Event, resp::RespError};

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

#[derive(serde::Deserialize)]
pub(crate) struct DataBaseEvent {
    pub event: Event,
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
}

#[derive(serde::Serialize)]
pub(crate) struct DataBaseEventRef<'a> {
    pub event: &'a Event,
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
}

pub(crate) trait Database {
    fn get_message(&self, message_id: &str) -> Option<DataBaseEvent>;
    fn insert_message(&self, value: &Event, seqs: Vec<i32>, rands: Vec<i32>);
    fn get_image<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &[u8],
    ) -> Result<Option<T>, RespError>;
    fn insert_image<T: serde::Serialize + SImage>(&self, value: &T);
    fn get_voice<T: SVoice>(&self, key: &[u8]) -> Result<Option<T>, RespError>;
    fn insert_voice<T: SVoice>(&self, value: &T);
}

pub(crate) enum WQDatabaseInner {
    SledDb(sleddb::SledDb),
    LevelDb(leveldb::LevelDb),
}

impl Database for WQDatabaseInner {
    fn get_message(&self, key: &str) -> Option<DataBaseEvent> {
        match self {
            Self::SledDb(db) => db.get_message(key),
            Self::LevelDb(db) => db.get_message(key),
        }
    }
    fn insert_message(&self, value: &Event, seqs: Vec<i32>, rands: Vec<i32>) {
        match self {
            Self::SledDb(db) => db.insert_message(value, seqs, rands),
            Self::LevelDb(db) => db.insert_message(value, seqs, rands),
        }
    }
    fn get_image<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &[u8],
    ) -> Result<Option<T>, RespError> {
        match self {
            Self::SledDb(db) => db.get_image(key),
            Self::LevelDb(db) => db.get_image(key),
        }
    }
    fn insert_image<T: serde::Serialize + SImage>(&self, value: &T) {
        match self {
            Self::SledDb(db) => db.insert_image(value),
            Self::LevelDb(db) => db.insert_image(value),
        }
    }
    fn get_voice<T: SVoice>(&self, key: &[u8]) -> Result<Option<T>, RespError> {
        match self {
            Self::SledDb(db) => db.get_voice(key),
            Self::LevelDb(db) => db.get_voice(key),
        }
    }
    fn insert_voice<T: SVoice>(&self, value: &T) {
        match self {
            Self::SledDb(db) => db.insert_voice(value),
            Self::LevelDb(db) => db.insert_voice(value),
        }
    }
}

// insert all but read the first
#[derive(Default)]
pub struct WQDatabase(Vec<WQDatabaseInner>);

impl WQDatabase {
    pub fn add_sled(mut self) -> Self {
        self.0.push(WQDatabaseInner::SledDb(sleddb::SledDb::init()));
        self
    }
    pub fn add_level(mut self) -> Self {
        self.0
            .push(WQDatabaseInner::LevelDb(leveldb::LevelDb::init()));
        self
    }
    pub fn not_empty(&self) -> bool {
        !self.0.is_empty()
    }
}

impl Database for WQDatabase {
    fn insert_message(&self, value: &Event, seqs: Vec<i32>, rands: Vec<i32>) {
        for db in &self.0 {
            db.insert_message(value, seqs.clone(), rands.clone())
        }
    }
    fn get_message(&self, key: &str) -> Option<DataBaseEvent> {
        for db in &self.0 {
            match db.get_message(key) {
                Some(v) => return Some(v),
                None => continue,
            }
        }
        None
    }
    fn insert_image<T: serde::Serialize + SImage>(&self, value: &T) {
        for db in &self.0 {
            db.insert_image(value)
        }
    }
    fn get_image<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &[u8],
    ) -> Result<Option<T>, RespError> {
        for db in &self.0 {
            match db.get_image(key)? {
                Some(v) => return Ok(Some(v)),
                None => continue,
            }
        }
        Ok(None)
    }
    fn insert_voice<T: SVoice>(&self, value: &T) {
        for db in &self.0 {
            db.insert_voice(value)
        }
    }
    fn get_voice<T: SVoice>(&self, key: &[u8]) -> Result<Option<T>, RespError> {
        for db in &self.0 {
            match db.get_voice(key)? {
                Some(v) => return Ok(Some(v)),
                None => continue,
            }
        }
        Ok(None)
    }
}
