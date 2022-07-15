use std::path::PathBuf;

use crate::error;
use prost::Message;
use ricq_core::pb::msg::Ptt;
use serde::{Deserialize, Serialize};
use walle_core::resp::RespError;
use walle_core::structs::FileId;

pub async fn save_voice(data: &[u8]) -> Result<LocalVoice, RespError> {
    use tokio::io::AsyncWriteExt;

    let md5 = md5::compute(data).0.to_vec();
    let size = data.len() as u32;
    let local = LocalVoice { md5, size };
    let mut file = tokio::fs::File::create(&local.path())
        .await
        .map_err(error::file_create_error)?;
    file.write_all(data.as_ref())
        .await
        .map_err(error::file_write_error)?;
    Ok(local)
}

pub trait SVoice: Sized {
    fn get_md5(&self) -> &[u8];
    fn get_size(&self) -> u32;
    fn to_data(&self) -> Vec<u8>;
    fn from_data(data: &Vec<u8>) -> Option<Self>;
    fn voice_id(&self) -> Vec<u8> {
        [self.get_md5(), self.get_size().to_be_bytes().as_slice()].concat()
    }
    fn hex_voice_id(&self) -> String {
        hex::encode(self.voice_id())
    }
    fn as_file_id_content(&self) -> FileId {
        FileId {
            file_id: self.hex_voice_id(),
        }
    }
}

impl SVoice for Ptt {
    fn get_md5(&self) -> &[u8] {
        self.file_md5()
    }
    fn get_size(&self) -> u32 {
        self.file_size() as u32
    }
    fn to_data(&self) -> Vec<u8> {
        self.encode_to_vec()
    }
    fn from_data(data: &Vec<u8>) -> Option<Self> {
        Message::decode(&data[..]).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalVoice {
    pub md5: Vec<u8>,
    pub size: u32,
}

impl SVoice for LocalVoice {
    fn get_md5(&self) -> &[u8] {
        &self.md5[..]
    }
    fn get_size(&self) -> u32 {
        self.size
    }
    fn to_data(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }
    fn from_data(data: &Vec<u8>) -> Option<Self> {
        rmp_serde::from_slice(&data[..]).ok()
    }
}

impl LocalVoice {
    pub fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(crate::VOICE_CACHE_DIR);
        path.push(self.hex_voice_id());
        path
    }
}

pub enum Voices {
    Local(LocalVoice),
    Ptt(Ptt),
}

impl SVoice for Voices {
    fn get_md5(&self) -> &[u8] {
        match self {
            Voices::Local(v) => v.get_md5(),
            Voices::Ptt(v) => v.get_md5(),
        }
    }
    fn get_size(&self) -> u32 {
        match self {
            Voices::Local(v) => v.get_size(),
            Voices::Ptt(v) => v.get_size(),
        }
    }
    fn to_data(&self) -> Vec<u8> {
        match self {
            Voices::Local(v) => v.to_data(),
            Voices::Ptt(v) => v.to_data(),
        }
    }
    fn from_data(data: &Vec<u8>) -> Option<Self> {
        if let Some(v) = Ptt::from_data(data) {
            Some(Voices::Ptt(v))
        } else if let Some(v) = LocalVoice::from_data(data) {
            Some(Voices::Local(v))
        } else {
            None
        }
    }
}
