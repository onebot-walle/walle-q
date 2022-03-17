use async_trait::async_trait;
use bytes::Bytes;
use rq_engine::command::img_store::GroupImageStoreResp;
use rq_engine::command::long_conn::OffPicUpResp;
use rs_qq::msg::elem::{GroupImage, PrivateImage};
use rs_qq::structs::ImageInfo;
use rs_qq::Client;
use std::{io::Read, path::PathBuf};

const IMAGE_CACHE_DIR: &str = "./data/image";

/// FriendImage GroupImage ImageInfo(LocalImage)
#[async_trait]
pub trait SImage: Sized {
    fn image_id(&self) -> Vec<u8>;
    async fn data(&self) -> Option<Bytes>;
    async fn try_into_group_elem(&self, cli: &Client, target: i64) -> Option<GroupImage>;
    async fn try_into_private_elem(&self, cli: &Client, group_code: i64) -> Option<PrivateImage>;
    fn hex_image_id(&self) -> String {
        hex::encode(self.image_id())
    }
    fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(IMAGE_CACHE_DIR);
        path.push(self.hex_image_id());
        path
    }
}

fn local_image_data<T: SImage>(image: &T) -> Option<Bytes> {
    if let Ok(mut file) = std::fs::File::open(image.path()) {
        let mut data = Vec::new();
        file.read_to_end(&mut data).ok();
        Some(Bytes::from(data))
    } else {
        None
    }
}

#[async_trait]
impl SImage for PrivateImage {
    fn image_id(&self) -> Vec<u8> {
        [
            self.md5.as_slice(),
            (self.size as u32).to_be_bytes().as_slice(),
        ]
        .concat()
    }
    async fn data(&self) -> Option<Bytes> {
        match local_image_data(self) {
            Some(data) => Some(data),
            None => match crate::utils::get_data_by_http(&self.url(), [].into()).await {
                Ok(data) => Some(data),
                Err(_) => None,
            },
        }
    }
    async fn try_into_private_elem(&self, _cli: &Client, _target: i64) -> Option<PrivateImage> {
        Some(self.clone())
    }
    async fn try_into_group_elem(&self, cli: &Client, target: i64) -> Option<GroupImage> {
        if let Some(data) = self.data().await {
            cli.upload_group_image(target, data.to_vec()).await.ok()
        } else {
            None
        }
    }
}

#[async_trait]
impl SImage for GroupImage {
    fn image_id(&self) -> Vec<u8> {
        [
            self.md5.as_slice(),
            (self.size as u32).to_be_bytes().as_slice(),
        ]
        .concat()
    }
    async fn data(&self) -> Option<Bytes> {
        match local_image_data(self) {
            Some(data) => Some(data),
            None => match crate::utils::get_data_by_http(&self.url(), [].into()).await {
                Ok(data) => Some(data),
                Err(_) => None,
            },
        }
    }
    async fn try_into_private_elem(&self, cli: &Client, target: i64) -> Option<PrivateImage> {
        if let Some(data) = self.data().await {
            cli.upload_private_image(target, data.to_vec()).await.ok() // todo
        } else {
            None
        }
    }
    async fn try_into_group_elem(&self, _cli: &Client, _target: i64) -> Option<GroupImage> {
        Some(self.clone())
    }
}

#[async_trait]
impl SImage for ImageInfo {
    fn image_id(&self) -> Vec<u8> {
        [self.md5.as_slice(), self.size.to_be_bytes().as_slice()].concat()
    }
    async fn data(&self) -> Option<Bytes> {
        local_image_data(self)
    }
    async fn try_into_private_elem(&self, cli: &Client, target: i64) -> Option<PrivateImage> {
        match cli.get_private_image_store(target, self).await {
            Ok(r) => match r {
                OffPicUpResp::Exist(res_id) => Some(self.clone().into_private_image(res_id)),
                OffPicUpResp::UploadRequired {
                    res_id,
                    upload_key,
                    upload_addrs,
                } => {
                    if let Some(data) = self.data().await {
                        cli._upload_private_image(upload_key, upload_addrs, data.to_vec())
                            .await
                            .and_then(|_| Ok(self.clone().into_private_image(res_id)))
                            .ok() // todo
                    } else {
                        tracing::warn!("image data is none");
                        None
                    }
                }
            },
            Err(e) => {
                tracing::warn!(
                    target: crate::WALLE_Q,
                    "get_private_image_store error {:?}",
                    e
                );
                None
            }
        }
    }
    async fn try_into_group_elem(&self, cli: &Client, target: i64) -> Option<GroupImage> {
        match cli.get_group_image_store(target, self).await {
            Ok(r) => match r {
                GroupImageStoreResp::Exist { file_id } => {
                    Some(self.clone().into_group_image(file_id))
                }
                GroupImageStoreResp::NotExist {
                    file_id,
                    upload_key,
                    upload_addrs,
                } => {
                    if let Some(data) = self.data().await {
                        cli._upload_group_image(upload_key, upload_addrs, data.to_vec())
                            .await
                            .and_then(|_| Ok(self.clone().into_group_image(file_id)))
                            .ok() // todo
                    } else {
                        tracing::warn!("image data is none");
                        None
                    }
                }
            },
            Err(e) => {
                tracing::warn!(
                    target: crate::WALLE_Q,
                    "get_private_image_store error {:?}",
                    e
                );
                None
            }
        }
    }
}
