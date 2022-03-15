use std::path::PathBuf;

use rq_engine::{
    command::{img_store::GroupImageStoreResp, long_conn::OffPicUpResp},
    msg::elem::{FriendImage, GroupImage},
    structs::{GroupMessage, MessageReceipt, PrivateMessage},
    RQError, RQResult,
};
use rs_qq::{structs::ImageInfo, Client};
use serde::{Deserialize, Serialize};
use walle_core::{resp::FileIdContent, Message};

pub(crate) mod sleddb;

const IMAGE_CACHE_DIR: &str = "./data/image";

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database: DatabaseInit + Sized {
    fn _get_message<T: for<'de> serde::Deserialize<'de>>(&self, key: i32) -> Option<T>;
    fn _insert_message<T: serde::Serialize + MessageId>(&self, value: &T);
    fn _get_image<T: for<'de> serde::Deserialize<'de>>(&self, key: &str) -> Option<T>;
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
    fn get_image(&self, key: &str) -> Option<SImage> {
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
    fn image_id(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SImage {
    pub md5: Vec<u8>, // also use as id
    pub width: u32,
    pub height: u32,
    pub image_type: i32,
    pub size: u32,
    pub filename: String,
    pub format: u8,
}

impl ImageId for SImage {
    fn image_id(&self) -> &str {
        self.filename.as_str()
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
            format: info.format.to_u8(),
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
            format: image::ImageFormat::from_u8(image.format),
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

    pub fn data(&self) -> RQResult<Vec<u8>> {
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
            file_id: self.image_id().to_string(),
        }
    }

    pub async fn try_into_private_elem(self, cli: &Client, target: i64) -> RQResult<FriendImage> {
        let info: ImageInfo = self.into();
        match cli.get_private_image_store(target, &info).await? {
            OffPicUpResp::Exist(image_id) => Ok(info.into_friend_image(image_id)),
            _ => Err(RQError::Other(
                crate::parse::err::IMAGE_NOT_EXIST.to_string(),
            )),
        }
    }

    pub async fn try_into_group_elem(self, cli: &Client, group_code: i64) -> RQResult<GroupImage> {
        let info: ImageInfo = self.into();
        match cli.get_group_image_store(group_code, &info).await? {
            GroupImageStoreResp::Exist { file_id } => Ok(info.into_group_image(file_id)),
            _ => Err(RQError::Other(
                crate::parse::err::IMAGE_NOT_EXIST.to_string(),
            )),
        }
    }
}

pub trait U8Enum {
    fn to_u8(&self) -> u8;
    fn from_u8(v: u8) -> Self;
}

impl U8Enum for image::ImageFormat {
    fn to_u8(&self) -> u8 {
        match self {
            Self::Png => 0,
            Self::Jpeg => 1,
            Self::Gif => 2,
            Self::WebP => 3,
            Self::Pnm => 4,
            Self::Tiff => 5,
            Self::Tga => 6,
            Self::Dds => 7,
            Self::Bmp => 8,
            Self::Ico => 9,
            Self::Hdr => 10,
            Self::OpenExr => 11,
            Self::Farbfeld => 12,
            Self::Avif => 13,
            _ => 0,
        }
    }

    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Png,
            1 => Self::Jpeg,
            2 => Self::Gif,
            3 => Self::WebP,
            4 => Self::Pnm,
            5 => Self::Tiff,
            6 => Self::Tga,
            7 => Self::Dds,
            8 => Self::Bmp,
            9 => Self::Ico,
            10 => Self::Hdr,
            11 => Self::OpenExr,
            12 => Self::Farbfeld,
            13 => Self::Avif,
            _ => Self::Png,
        }
    }
}
