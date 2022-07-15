use std::path::PathBuf;

use cached::Cached;
use sha2::Digest;
use tokio::io::AsyncWriteExt;
use tokio::{fs::File, io::AsyncReadExt};
use walle_core::action::{GetFileFragmented, UploadFileFragmented};
use walle_core::extended_value;
use walle_core::resp::{Resp, RespError};
use walle_core::structs::FileId;
use walle_core::util::OneBotBytes;

use crate::database::{save_image, save_voice, Database, Images, SImage, SVoice};
use crate::error;
use crate::model::{WQGetFile, WQUploadFile};

use super::RespResult;

impl super::Handler {
    async fn get_file_data(c: WQUploadFile) -> Result<Vec<u8>, RespError> {
        match c.ty.as_str() {
            "url" if let Some(url) = c.url => {
                uri_reader::uget_with_headers(&url, c.headers.unwrap_or_default())
                    .await
                    .map_err(|e|error::net_download_fail(e))
            }
            "path" if let Some(path) = c.path => {
                let input_path = PathBuf::from(path);
                let mut file =  File::open(&input_path).await.map_err(|e| {
                    error::file_open_error(e)
                })?;
                let mut data = Vec::new();
                file.read_to_end(&mut data).await.map_err(|e| {
                    error::file_read_error(e)
                })?;
                Ok(data)
            }
            "data" if let Some(data) = c.data => Ok(data.0),
            ty => Err(error::unsupported_param(ty)),
        }
    }

    pub async fn upload_file(&self, c: WQUploadFile) -> RespResult<FileId> {
        let file_type = c.file_type.clone().unwrap_or("image".to_string());
        let data = Self::get_file_data(c).await?;
        match file_type.as_str() {
            "image" => self.upload_image(data).await,
            "voice" => self.upload_voice(data).await,
            ty => Err(error::unsupported_param(ty)),
        }
    }

    pub async fn upload_image(&self, data: Vec<u8>) -> RespResult<FileId> {
        let info = save_image(&data).await?;
        self.database.insert_image(&info);
        Ok(info.as_file_id_content())
    }

    pub async fn upload_voice(&self, data: Vec<u8>) -> RespResult<FileId> {
        let local = save_voice(&data).await?;
        self.database.insert_voice(&local);
        Ok(local.as_file_id_content())
    }

    pub async fn get_file(&self, c: WQGetFile) -> RespResult<WQUploadFile> {
        let file_type = c.file_type.clone().unwrap_or("image".to_string());
        match file_type.as_str() {
            "image" => self.get_image(&c).await,
            ty => Err(error::unsupported_param(ty)),
        }
    }

    pub async fn get_image(&self, c: &WQGetFile) -> RespResult<WQUploadFile> {
        if let Some(image) = self.database.get_image::<Images>(
            &hex::decode(&c.file_id).map_err(|_| error::bad_param("file_id"))?,
        )? {
            match c.ty.as_str() {
                "url" => {
                    if let Some(url) = image.get_url() {
                        Ok(WQUploadFile {
                            ty: "url".to_string(),
                            name: image.get_file_name().to_string(),
                            url: Some(url),
                            headers: None,
                            path: None,
                            data: None,
                            sha256: None,
                            file_type: Some("image".to_string()),
                        })
                    } else {
                        Err(error::bad_image_url(image.get_file_name()))
                    }
                }
                "path" => {
                    if image.path().exists() {
                        Ok(WQUploadFile {
                            ty: "path".to_string(),
                            name: image.get_file_name().to_string(),
                            path: Some(image.path().to_str().unwrap().to_string()),
                            url: None,
                            headers: None,
                            data: None,
                            sha256: None,
                            file_type: Some("image".to_string()),
                        })
                    } else {
                        Err(error::bad_image_path(image.get_file_name()))
                    }
                }
                "data" => {
                    if let Ok(data) = image.data().await {
                        Ok(WQUploadFile {
                            ty: "data".to_string(),
                            name: image.get_file_name().to_string(),
                            data: Some(data.into()),
                            url: None,
                            path: None,
                            headers: None,
                            sha256: None,
                            file_type: Some("image".to_string()),
                        })
                    } else {
                        Err(error::bad_image_data(image.get_file_name()))
                    }
                }
                ty => Err(error::unsupported_param(ty)),
            }
        } else {
            Err(error::file_not_found(&c.file_id))
        }
    }

    pub async fn upload_file_fragmented(
        &self,
        c: UploadFileFragmented,
    ) -> RespResult<Option<FileId>> {
        match c {
            UploadFileFragmented::Prepare {
                name, total_size, ..
            } => {
                let file_id = format!("{}-{}", name, total_size);
                self.uploading_fragment.lock().await.cache_set(
                    file_id.clone(),
                    FragmentFile {
                        total_size,
                        files: vec![],
                    },
                );
                Ok(Some(FileId { file_id }))
            }
            UploadFileFragmented::Transfer {
                file_id,
                offset,
                size,
                data,
                ..
            } => {
                let mut file_path = std::path::PathBuf::from(crate::FILE_CACHE_DIR);
                file_path.push(format!("{}-{}", file_id, offset));
                let mut file = tokio::fs::File::create(file_path)
                    .await
                    .map_err(error::file_create_error)?;
                file.write_all(&data.0)
                    .await
                    .map_err(error::file_write_error)?;
                match self.uploading_fragment.lock().await.cache_get_mut(&file_id) {
                    Some(f) => f.files.push((offset, size)),
                    None => return Err(error::prepare_file_first(&file_id)),
                }
                Ok(None)
            }
            UploadFileFragmented::Finish {
                file_id, sha256, ..
            } => {
                let mut fragment = self
                    .uploading_fragment
                    .lock()
                    .await
                    .cache_remove(&file_id)
                    .ok_or_else(|| error::prepare_file_first(&file_id))?;
                fragment.files.sort();
                let mut data = Vec::with_capacity(fragment.total_size as usize);
                let mut total_size = 0;
                for (offset, size) in fragment.files {
                    let mut file_path = std::path::PathBuf::from(crate::FILE_CACHE_DIR);
                    file_path.push(format!("{}-{}", file_id, offset));
                    let mut file = tokio::fs::File::open(&file_path)
                        .await
                        .map_err(error::file_open_error)?;
                    file.read_buf(&mut data)
                        .await
                        .map_err(error::file_read_error)?;
                    drop(file);
                    tokio::fs::remove_file(file_path).await.ok();
                    total_size += size;
                }
                if total_size != fragment.total_size {
                    return Err(error::file_total_size_not_match(format!(
                        "get {} of {}",
                        total_size, fragment.total_size
                    )));
                }
                let mut calc_sha = sha2::Sha256::default();
                calc_sha.update(&data);
                let calc_sha = calc_sha.finalize().to_vec();
                if let Some(sha) = sha256 {
                    let sha = hex::decode(sha).map_err(|_| error::bad_param("sha256"))?;
                    if calc_sha != sha {
                        return Err(error::file_sha256_not_match(format!(
                            "get {} of {}",
                            hex::encode(calc_sha),
                            hex::encode(sha)
                        )));
                    }
                }
                self.upload_image(data).await.map(|id| Some(id))
            }
        }
    }

    pub async fn get_file_fragmented(&self, c: GetFileFragmented) -> RespResult<Resp> {
        use ricq::structs::ImageInfo;
        use tokio::io::{AsyncSeekExt, SeekFrom};
        async fn to_info(
            h: &super::Handler,
            simage: impl SImage,
        ) -> Result<(ImageInfo, String), RespError> {
            let data = simage.data().await.map_err(error::rq_error)?;
            let sha256 = {
                let mut s = sha2::Sha256::default();
                s.update(&data);
                hex::encode(&s.finalize())
            };
            let info = save_image(&data).await?;
            h.database.insert_image(&info);
            Ok((info, sha256))
        }
        match c {
            GetFileFragmented::Prepare { file_id, .. } => {
                let (info, sha256) = match self
                    .database
                    .get_image(&hex::decode(&file_id).map_err(|_| error::bad_param("file_id"))?)?
                    .ok_or_else(|| error::file_not_found(&file_id))?
                {
                    Images::Friend(f) => to_info(self, f).await?,
                    Images::Group(g) => to_info(self, g).await?,
                    Images::Info(i) => {
                        let data = i.data().await.map_err(error::rq_error)?;
                        let sha256 = {
                            let mut s = sha2::Sha256::default();
                            s.update(&data);
                            hex::encode(&s.finalize())
                        };
                        (i, sha256)
                    }
                };
                Ok(extended_value!({
                    "name": info.filename,
                    "total_size": info.size,
                    "sha256": sha256
                })
                .into())
            }
            GetFileFragmented::Transfer {
                file_id,
                offset,
                size,
                ..
            } => {
                let info: ImageInfo = self
                    .database
                    .get_image(&hex::decode(&file_id).map_err(|_| error::bad_param("file_id"))?)?
                    .ok_or_else(|| error::file_not_found(&file_id))?;
                let mut file = tokio::fs::File::open(info.path())
                    .await
                    .map_err(error::file_open_error)?;
                file.seek(SeekFrom::Start(offset as u64))
                    .await
                    .map_err(error::file_read_error)?;
                let mut data = Vec::with_capacity(size as usize);
                file.read(&mut data).await.map_err(error::file_read_error)?;
                let data = OneBotBytes(data);
                Ok(extended_value!({ "data": data }).into())
            }
        }
    }
}

pub struct FragmentFile {
    pub total_size: i64,
    pub files: Vec<(i64, i64)>,
}
