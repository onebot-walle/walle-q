use std::sync::atomic::{AtomicUsize, Ordering};

use rusty_leveldb::{Options, DB};

use super::{Database, DatabaseInit, MessageId, SImage};

const MEM_CACHE_LIMIT: usize = 10;

pub(crate) struct LevelDb(std::sync::Mutex<DB>, AtomicUsize);

impl DatabaseInit for LevelDb {
    fn init() -> Self {
        let opt = Options::default();
        Self(
            std::sync::Mutex::new(DB::open("./data/leveldb", opt).unwrap()),
            AtomicUsize::new(0),
        )
    }
}

impl LevelDb {
    fn flush(&self, mut db: std::sync::MutexGuard<DB>) {
        if self.1.load(Ordering::Relaxed) > MEM_CACHE_LIMIT {
            tracing::debug!(target: crate::WALLE_Q, "Flushing leveldb cache");
            db.flush().unwrap();
            self.1.store(0, Ordering::Relaxed);
        } else {
            self.1.fetch_add(1, Ordering::Relaxed);
        }
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
        let mut db = self.0.lock().unwrap();
        db.put(
            &value.seq().to_be_bytes(),
            &rmp_serde::to_vec(value).unwrap(),
        )
        .unwrap();
        self.flush(db);
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
        T: serde::Serialize + SImage,
    {
        let mut db = self.0.lock().unwrap();
        db.put(&value.image_id(), &rmp_serde::to_vec(value).unwrap())
            .unwrap();
        self.flush(db);
    }
}
