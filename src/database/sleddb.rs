use super::*;
use sled::Tree;

pub(crate) struct SledDb {
    pub message_tree: Tree,
}

impl DatabaseInit for SledDb {
    fn init() -> Self {
        let s = sled::open("./data/sled").unwrap();
        Self {
            message_tree: s.open_tree("message").unwrap(),
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
            .insert(
                value.seq().to_be_bytes(),
                rmp_serde::to_vec(value).unwrap(),
            )
            .unwrap();
    }
}
