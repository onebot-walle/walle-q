use crate::error;

use super::*;
use sled::Tree;

pub(crate) struct SledDb {
    pub message_tree: Tree,
    pub image_tree: Tree,
    pub audio_tree: Tree,
}

impl DatabaseInit for SledDb {
    fn init(base_path: &str) -> Self {
        let s = sled::open(format!("{}/{}", base_path, "sled")).unwrap();
        Self {
            message_tree: s.open_tree("message").unwrap(),
            image_tree: s.open_tree("image").unwrap(),
            audio_tree: s.open_tree("audio").unwrap(),
        }
    }
}

impl Database for SledDb {
    fn get_message(&self, key: &str) -> Option<DataBaseEvent> {
        self.message_tree
            .get(key.as_bytes())
            .unwrap()
            .map(|v| rmp_serde::from_slice(&v).unwrap())
    }

    fn insert_message(&self, value: &Event, seqs: Vec<i32>, rands: Vec<i32>) {
        self.message_tree
            .insert(
                value.message_id().as_bytes(),
                rmp_serde::to_vec(&DataBaseEventRef {
                    event: value,
                    seqs,
                    rands,
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn get_image<T>(&self, key: &[u8]) -> Result<Option<T>, RespError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.image_tree
            .get(key)
            .unwrap()
            .map(|v| rmp_serde::from_slice(&v).map_err(|e| error::file_type_not_match(e)))
            .transpose()
    }

    fn insert_image<T>(&self, value: &T)
    where
        T: serde::Serialize + SImage,
    {
        self.image_tree
            .insert(value.image_id(), rmp_serde::to_vec(value).unwrap())
            .unwrap();
    }
    fn get_voice<T: SVoice>(&self, key: &[u8]) -> Result<Option<T>, RespError> {
        self.audio_tree
            .get(key)
            .unwrap()
            .map(|v| SVoice::from_data(&v.to_vec()).ok_or_else(|| error::file_type_not_match("")))
            .transpose()
    }
    fn insert_voice<T: SVoice>(&self, value: &T) {
        self.audio_tree
            .insert(value.voice_id(), value.to_data())
            .unwrap();
    }
}
