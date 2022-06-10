pub(crate) mod leveldb;
pub(crate) mod message;
pub(crate) mod simage;
pub(crate) mod sleddb;
pub(crate) mod voice;

pub use message::*;
pub use simage::*;
pub use voice::*;
use walle_core::resp::RespError;

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database {
    fn _get_message<T: for<'de> serde::Deserialize<'de>>(&self, key: i32) -> Option<T>;
    fn _insert_message<T: serde::Serialize + MessageId>(&self, value: &T);
    fn get_image<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &[u8],
    ) -> Result<Option<T>, RespError>;
    fn insert_image<T: serde::Serialize + SImage>(&self, value: &T);
    fn get_voice<T: SVoice>(&self, key: &[u8]) -> Result<Option<T>, RespError>;
    fn insert_voice<T: SVoice>(&self, value: &T);
    fn get_message(&self, key: i32) -> Option<SMessage> {
        self._get_message(key)
    }
    fn get_group_message(&self, key: i32) -> Option<SGroupMessage> {
        self._get_message(key)
    }
    fn insert_group_message(&self, value: &SGroupMessage) {
        self._insert_message(value)
    }
    fn get_private_message(&self, key: i32) -> Option<SPrivateMessage> {
        self._get_message(key)
    }
    fn insert_private_message(&self, value: &SPrivateMessage) {
        self._insert_message(value)
    }
}

pub(crate) enum WQDatabaseInner {
    SledDb(sleddb::SledDb),
    LevelDb(leveldb::LevelDb),
}

impl Database for WQDatabaseInner {
    fn _get_message<T: for<'de> serde::Deserialize<'de>>(&self, key: i32) -> Option<T> {
        match self {
            Self::SledDb(db) => db._get_message(key),
            Self::LevelDb(db) => db._get_message(key),
        }
    }
    fn _insert_message<T: serde::Serialize + MessageId>(&self, value: &T) {
        match self {
            Self::SledDb(db) => db._insert_message(value),
            Self::LevelDb(db) => db._insert_message(value),
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
pub(crate) struct WQDatabase(pub(crate) Vec<WQDatabaseInner>);

impl WQDatabase {
    pub(crate) fn sled(mut self) -> Self {
        self.0.push(WQDatabaseInner::SledDb(sleddb::SledDb::init()));
        self
    }
    pub(crate) fn level(mut self) -> Self {
        self.0
            .push(WQDatabaseInner::LevelDb(leveldb::LevelDb::init()));
        self
    }
}

impl Database for WQDatabase {
    fn _insert_message<T: serde::Serialize + MessageId>(&self, value: &T) {
        for db in &self.0 {
            db._insert_message(value)
        }
    }
    fn _get_message<T: for<'de> serde::Deserialize<'de>>(&self, key: i32) -> Option<T> {
        for db in &self.0 {
            match db._get_message(key) {
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
