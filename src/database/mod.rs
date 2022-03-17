use std::path::PathBuf;

use crate::parse::err::*;
use rq_engine::{
    command::{img_store::GroupImageStoreResp, long_conn::OffPicUpResp},
    msg::elem::{PrivateImage, GroupImage},
    structs::{GroupMessage, MessageReceipt, PrivateMessage},
    RQResult,
};
use rs_qq::{structs::ImageInfo, Client};
use serde::{Deserialize, Serialize};
use walle_core::{resp::FileIdContent, Message};

pub(crate) mod image;
pub(crate) mod sleddb;

const IMAGE_CACHE_DIR: &str = "./data/image";

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database: DatabaseInit + Sized {
    fn _get_message<T: for<'de> serde::Deserialize<'de>>(&self, key: i32) -> Option<T>;
    fn _insert_message<T: serde::Serialize + MessageId>(&self, value: &T);
    fn _get_image<T: for<'de> serde::Deserialize<'de>>(&self, key: &[u8]) -> Option<T>;
    fn _insert_image<T: serde::Serialize + ImageId>(&self, value: &T);
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
    fn get_image(&self, key: &[u8]) -> Option<SImage> {
        self._get_image(key)
    }
    fn insert_image(&self, value: &SImage) {
        self._insert_image(value)
    }
}

pub trait MessageId {
    fn seq(&self) -> i32;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SMessage {
    Group(SGroupMessage),
    Private(SPrivateMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SGroupMessage {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub group_code: i64,
    pub from_uin: i64,
    pub time: i32,
    pub message: Message,
}

impl MessageId for SGroupMessage {
    fn seq(&self) -> i32 {
        self.seqs[0]
    }
}

impl SGroupMessage {
    pub fn new(m: GroupMessage, message: Message) -> Self {
        Self {
            seqs: m.seqs,
            rands: m.rands,
            group_code: m.group_code,
            from_uin: m.from_uin,
            time: m.time,
            message,
        }
    }

    pub fn receipt(
        receipt: MessageReceipt,
        group_code: i64,
        from_uin: i64,
        message: Message,
    ) -> Self {
        Self {
            seqs: receipt.seqs,
            rands: receipt.rands,
            group_code,
            from_uin,
            time: receipt.time as i32,
            message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SPrivateMessage {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub target: i64,
    pub time: i32,
    pub from_uin: i64,
    pub from_nick: String,
    pub message: Message,
}

impl MessageId for SPrivateMessage {
    fn seq(&self) -> i32 {
        self.seqs[0]
    }
}

impl SPrivateMessage {
    pub fn new(m: PrivateMessage, message: Message) -> Self {
        Self {
            seqs: m.seqs,
            rands: m.rands,
            target: m.target,
            from_uin: m.from_uin,
            from_nick: m.from_nick,
            time: m.time,
            message,
        }
    }

    pub fn receipt(
        receipt: MessageReceipt,
        target: i64,
        from_uin: i64,
        from_nick: String,
        message: Message,
    ) -> Self {
        Self {
            seqs: receipt.seqs,
            rands: receipt.rands,
            target,
            from_uin,
            from_nick,
            time: receipt.time as i32,
            message,
        }
    }
}

pub trait ImageId {
    fn image_id(&self) -> Vec<u8>;
    fn hex_image_id(&self) -> String {
        hex::encode(self.image_id())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SImage {
    pub md5: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub image_type: i32,
    pub size: u32,
    pub filename: String,
}

impl ImageId for SImage {
    fn image_id(&self) -> Vec<u8> {
        [self.md5.as_slice(), self.size.to_be_bytes().as_slice()].concat()
    }
}

impl From<ImageInfo> for SImage {
    fn from(info: ImageInfo) -> Self {
        Self {
            md5: info.md5,
            width: info.width,
            height: info.height,
            image_type: info.image_type,
            size: info.size,
            filename: info.filename,
        }
    }
}

impl From<GroupImage> for SImage {
    fn from(img: GroupImage) -> Self {
        Self {
            md5: img.md5,
            width: img.width as u32,
            height: img.height as u32,
            image_type: img.image_type,
            size: img.size as u32,
            filename: img.image_id,
        }
    }
}

impl From<SImage> for ImageInfo {
    fn from(image: SImage) -> Self {
        Self {
            md5: image.md5,
            width: image.width,
            height: image.height,
            image_type: image.image_type,
            size: image.size,
            filename: image.filename,
        }
    }
}

impl SImage {
    pub fn try_save(data: &[u8]) -> RQResult<Self> {
        use std::io::Write;
        let image: Self = ImageInfo::try_new(data)?.into();
        let mut file = std::fs::File::create(image.path())?;
        file.write_all(data)?;
        Ok(image)
    }

    pub fn data(&self) -> Result<Vec<u8>, std::io::Error> {
        use std::io::Read;
        let mut file = std::fs::File::open(self.path())?;
        let mut data = Vec::with_capacity(self.size as usize);
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    pub fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(IMAGE_CACHE_DIR);
        path.push(self.filename.clone());
        path
    }

    pub fn as_file_id_content(&self) -> FileIdContent {
        FileIdContent {
            file_id: self.hex_image_id(),
        }
    }

    pub async fn try_into_private_elem(self, cli: &Client, target: i64) -> WQResult<PrivateImage> {
        let info: ImageInfo = self.into();
        match cli.get_private_image_store(target, &info).await? {
            OffPicUpResp::Exist(image_id) => Ok(info.into_private_image(image_id)),
            _ => Err(WQError::image_not_exist()),
        }
    }

    pub async fn try_into_group_elem(self, cli: &Client, group_code: i64) -> WQResult<GroupImage> {
        let info: ImageInfo = self.into();
        match cli.get_group_image_store(group_code, &info).await? {
            GroupImageStoreResp::Exist { file_id } => Ok(info.into_group_image(file_id)),
            _ => Err(WQError::image_not_exist()),
        }
    }
}
