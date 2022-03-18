use rusty_leveldb::{Options, DB};

use super::{Database, DatabaseInit, ImageId, MessageId};

pub(crate) struct LevelDb(std::sync::Mutex<DB>);

impl DatabaseInit for LevelDb {
    fn init() -> Self {
        let opt = Options::default();
        Self(std::sync::Mutex::new(
            DB::open("./data/leveldb", opt).unwrap(),
        ))
    }
}

impl Database for LevelDb {
    fn _get_message<T>(&self, key: i32) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.0
            .lock()
            .unwrap()
            .get(&key.to_be_bytes())
            .and_then(|v| rmp_serde::from_read_ref(&v).unwrap())
    }
    fn _insert_message<T>(&self, value: &T)
    where
        T: serde::Serialize + MessageId,
    {
        self.0
            .lock()
            .unwrap()
            .put(
                &value.seq().to_be_bytes(),
                &rmp_serde::to_vec(value).unwrap(),
            )
            .unwrap();
    }
    fn _get_image<T>(&self, key: &[u8]) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.0
            .lock()
            .unwrap()
            .get(key)
            .and_then(|v| rmp_serde::from_read_ref(&v).unwrap())
    }
    fn _insert_image<T>(&self, value: &T)
    where
        T: serde::Serialize + ImageId,
    {
        self.0
            .lock()
            .unwrap()
            .put(&value.image_id(), &rmp_serde::to_vec(value).unwrap())
            .unwrap();
    }
}
