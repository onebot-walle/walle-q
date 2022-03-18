pub(crate) mod image;
pub(crate) mod leveldb;
pub(crate) mod message;
pub(crate) mod sleddb;

pub use image::*;
pub use message::*;

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database: DatabaseInit + Sized {
    fn _get_message<T: for<'de> serde::Deserialize<'de>>(&self, key: i32) -> Option<T>;
    fn _insert_message<T: serde::Serialize + MessageId>(&self, value: &T);
    fn _get_image<T: for<'de> serde::Deserialize<'de>>(&self, key: &[u8]) -> Option<T>;
    fn _insert_image<T: serde::Serialize + SImage>(&self, value: &T);
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
    fn get_image(&self, key: &[u8]) -> Option<Images> {
        self._get_image(key)
    }
    fn insert_image(&self, value: &Images) {
        self._insert_image(value)
    }
}
