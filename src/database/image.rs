use async_trait::async_trait;
use rq_engine::{RQError, RQResult};
use rs_qq::msg::elem::{FriendImage, GroupImage};
use rs_qq::structs::ImageInfo;
use rs_qq::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walle_core::resp::FileIdContent;

pub const IMAGE_CACHE_DIR: &str = "./data/image";

pub async fn save_image(data: &[u8]) -> Result<ImageInfo, &'static str> {
    use tokio::io::AsyncWriteExt;

    let info = ImageInfo::try_new(data).map_err(|_| "图片解析失败")?;
    let mut file = tokio::fs::File::create(&info.path())
        .await
        .map_err(|_| "文件创建失败")?;
    file.write_all(data.as_ref())
        .await
        .map_err(|_| "文件写入失败")?;
    Ok(info)
}

/// FriendImage GroupImage ImageInfo(LocalImage)
#[async_trait]
pub trait SImage: Sized {
    fn get_md5(&self) -> &[u8];
    fn get_size(&self) -> u32;
    async fn data(&self) -> RQResult<Vec<u8>>;
    async fn try_into_group_elem(&self, cli: &Client, target: i64) -> Option<GroupImage>;
    async fn try_into_friend_elem(&self, cli: &Client, group_code: i64) -> Option<FriendImage>;
    fn image_id(&self) -> Vec<u8> {
        [self.get_md5(), self.get_size().to_be_bytes().as_slice()].concat()
    }
    fn hex_image_id(&self) -> String {
        hex::encode(self.image_id())
    }
    fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(IMAGE_CACHE_DIR);
        path.push(self.hex_image_id());
        path
    }
    fn as_file_id_content(&self) -> FileIdContent {
        FileIdContent {
            file_id: self.hex_image_id(),
        }
    }
}

async fn local_image_data<T: SImage>(image: &T) -> Result<Vec<u8>, std::io::Error> {
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(image.path()).await?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).await?;
    Ok(data)
}

fn new_info_from_group(group_image: &GroupImage) -> ImageInfo {
    ImageInfo {
        md5: group_image.md5.clone(),
        width: group_image.width as u32,
        height: group_image.height as u32,
        image_type: group_image.image_type,
        size: group_image.size as u32,
        filename: group_image.file_path.clone(),
    }
}

#[async_trait]
impl SImage for FriendImage {
    fn get_md5(&self) -> &[u8] {
        self.md5.as_slice()
    }
    fn get_size(&self) -> u32 {
        self.size as u32
    }
    async fn data(&self) -> RQResult<Vec<u8>> {
        match local_image_data(self).await {
            Ok(data) => Ok(data),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    uri_reader::uget(&self.url())
                        .await
                        .map_err(|e| RQError::Other(e.to_string()))
                } else {
                    Err(e.into())
                }
            }
        }
    }
    async fn try_into_friend_elem(&self, _cli: &Client, _target: i64) -> Option<FriendImage> {
        Some(self.clone())
    }
    async fn try_into_group_elem(&self, cli: &Client, target: i64) -> Option<GroupImage> {
        if let Ok(data) = self.data().await {
            cli.upload_group_image(target, data.to_vec()).await.ok()
        } else {
            None
        }
    }
}

#[async_trait]
impl SImage for GroupImage {
    fn get_md5(&self) -> &[u8] {
        self.md5.as_slice()
    }
    fn get_size(&self) -> u32 {
        self.size as u32
    }
    async fn data(&self) -> RQResult<Vec<u8>> {
        match local_image_data(self).await {
            Ok(data) => Ok(data),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    uri_reader::uget(&self.url())
                        .await
                        .map_err(|e| RQError::Other(e.to_string()))
                } else {
                    Err(e.into())
                }
            }
        }
    }
    async fn try_into_friend_elem(&self, cli: &Client, target: i64) -> Option<FriendImage> {
        use rs_qq::ext::image::upload_friend_image_ext;
        let info = new_info_from_group(self);

        upload_friend_image_ext(cli, target, info, |info| {
            Box::pin(async { info.data().await })
        })
        .await
        .ok()
    }
    async fn try_into_group_elem(&self, _cli: &Client, _target: i64) -> Option<GroupImage> {
        Some(self.clone())
    }
}

#[async_trait]
impl SImage for ImageInfo {
    fn get_md5(&self) -> &[u8] {
        self.md5.as_slice()
    }
    fn get_size(&self) -> u32 {
        self.size
    }
    async fn data(&self) -> RQResult<Vec<u8>> {
        local_image_data(self).await.map_err(RQError::IO)
    }
    async fn try_into_friend_elem(&self, cli: &Client, target: i64) -> Option<FriendImage> {
        use rs_qq::ext::image::upload_friend_image_ext;
        upload_friend_image_ext(cli, target, self.clone(), |info| {
            Box::pin(async { info.data().await })
        })
        .await
        .ok()
    }
    async fn try_into_group_elem(&self, cli: &Client, target: i64) -> Option<GroupImage> {
        use rs_qq::ext::image::upload_group_image_ext;
        upload_group_image_ext(cli, target, self.clone(), |info| {
            Box::pin(async { info.data().await })
        })
        .await
        .ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Images {
    Friend(FriendImage),
    Group(GroupImage),
    Info(ImageInfo),
}

#[async_trait]
impl SImage for Images {
    fn get_md5(&self) -> &[u8] {
        match self {
            Images::Friend(image) => image.get_md5(),
            Images::Group(image) => image.get_md5(),
            Images::Info(image) => image.get_md5(),
        }
    }
    fn get_size(&self) -> u32 {
        match self {
            Images::Friend(image) => image.get_size(),
            Images::Group(image) => image.get_size(),
            Images::Info(image) => image.get_size(),
        }
    }
    async fn data(&self) -> RQResult<Vec<u8>> {
        match self {
            Images::Friend(image) => image.data().await,
            Images::Group(image) => image.data().await,
            Images::Info(image) => image.data().await,
        }
    }
    async fn try_into_friend_elem(&self, cli: &Client, target: i64) -> Option<FriendImage> {
        match self {
            Images::Friend(image) => image.try_into_friend_elem(cli, target).await,
            Images::Group(image) => image.try_into_friend_elem(cli, target).await,
            Images::Info(image) => image.try_into_friend_elem(cli, target).await,
        }
    }
    async fn try_into_group_elem(&self, cli: &Client, target: i64) -> Option<GroupImage> {
        match self {
            Images::Friend(image) => image.try_into_group_elem(cli, target).await,
            Images::Group(image) => image.try_into_group_elem(cli, target).await,
            Images::Info(image) => image.try_into_group_elem(cli, target).await,
        }
    }
}
