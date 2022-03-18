use super::*;
use sled::Tree;

pub(crate) struct SledDb {
    pub message_tree: Tree,
    pub image_tree: Tree,
}

impl DatabaseInit for SledDb {
    fn init() -> Self {
        let s = sled::open("./data/sled").unwrap();
        Self {
            message_tree: s.open_tree("message").unwrap(),
            image_tree: s.open_tree("image").unwrap(),
        }
    }
}

impl Database for SledDb {
    fn _get_message<T>(&self, key: i32) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.message_tree
            .get(key.to_be_bytes())
            .unwrap()
            .map(|v| rmp_serde::from_read_ref(&v).unwrap())
    }

    fn _insert_message<T>(&self, value: &T)
    where
        T: serde::Serialize + MessageId,
    {
        self.message_tree
            .insert(value.seq().to_be_bytes(), rmp_serde::to_vec(value).unwrap())
            .unwrap();
    }

    fn _get_image<T>(&self, key: &[u8]) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.image_tree
            .get(key)
            .unwrap()
            .map(|v| rmp_serde::from_read_ref(&v).unwrap())
    }

    fn _insert_image<T>(&self, value: &T)
    where
        T: serde::Serialize + SImage,
    {
        self.image_tree
            .insert(value.image_id(), rmp_serde::to_vec(value).unwrap())
            .unwrap();
    }
}
