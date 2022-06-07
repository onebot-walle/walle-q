use std::path::PathBuf;

use cached::Cached;
use sha2::Digest;
use tokio::io::AsyncWriteExt;
use tokio::{fs::File, io::AsyncReadExt};
use walle_core::action::{GetFile, GetFileFragmented, UploadFile, UploadFileFragmented};
use walle_core::resp::{FileFragmentedHead, FileIdContent, RespError};
use walle_core::{extended_map, ExtendedMap, ExtendedValue, Resps};

use crate::database::{save_image, Database, Images, SImage};
use crate::error;

use super::{OneBot, WQRespResult};

impl super::Handler {
    async fn get_file_data(c: UploadFile) -> Result<Vec<u8>, RespError> {
        match c.r#type.as_str() {
            "url" if let Some(url) = c.url => {
                uri_reader::uget_with_headers(&url, c.headers.unwrap_or_default())
                    .await
                    .map_err(|_|error::net_download_fail())
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
            "data" if let Some(data) = c.data => Ok(data),
            ty => Err(error::unsupported_param(ty)),
        }
    }

    pub async fn upload_file(&self, c: UploadFile, ob: &OneBot) -> WQRespResult {
        let file_type = c
            .extra
            .get("file_type")
            .ok_or_else(|| error::bad_param("file_type"))?
            .clone()
            .downcast_str()
            .map_err(|_| error::bad_param("file_type"))?;
        let data = Self::get_file_data(c).await?;
        match file_type.as_str() {
            "image" => self.upload_image(data, ob).await,
            ty => Err(error::unsupported_param(ty)),
        }
    }

    pub async fn upload_image(&self, data: Vec<u8>, _ob: &OneBot) -> WQRespResult {
        let info = save_image(&data).await?;
        self.database._insert_image(&info);
        Ok(Resps::success(info.as_file_id_content().into()))
    }

    pub async fn get_file(&self, c: GetFile, ob: &OneBot) -> WQRespResult {
        let file_type = c
            .extra
            .get("file_type")
            .ok_or_else(|| error::bad_param("file_type"))?
            .clone()
            .downcast_str()
            .map_err(|_| error::bad_param("file_type"))?;
        match file_type.as_str() {
            "image" => self.get_image(&c, ob).await,
            ty => Err(error::unsupported_param(ty)),
        }
    }

    pub async fn get_image(&self, c: &GetFile, _ob: &OneBot) -> WQRespResult {
        if let Some(image) = hex::decode(&c.file_id)
            .ok()
            .and_then(|id| self.database.get_image(&id))
        {
            match c.r#type.as_str() {
                "url" => {
                    if let Some(url) = image.get_url() {
                        Ok(Resps::success(
                            UploadFile {
                                r#type: "url".to_string(),
                                name: image.get_file_name().to_string(),
                                url: Some(url),
                                headers: None,
                                path: None,
                                data: None,
                                sha256: None,
                                extra: ExtendedMap::default(),
                            }
                            .into(),
                        ))
                    } else {
                        Err(error::bad_image_url(image.get_file_name()))
                    }
                }
                "path" => {
                    if image.path().exists() {
                        Ok(Resps::success(
                            UploadFile {
                                r#type: "path".to_string(),
                                name: image.get_file_name().to_string(),
                                path: Some(image.path().to_str().unwrap().to_string()),
                                url: None,
                                headers: None,
                                data: None,
                                sha256: None,
                                extra: ExtendedMap::default(),
                            }
                            .into(),
                        ))
                    } else {
                        Err(error::bad_image_path(image.get_file_name()))
                    }
                }
                "data" => {
                    if let Ok(data) = image.data().await {
                        Ok(Resps::success(
                            UploadFile {
                                r#type: "data".to_string(),
                                name: image.get_file_name().to_string(),
                                data: Some(data),
                                url: None,
                                path: None,
                                headers: None,
                                sha256: None,
                                extra: ExtendedMap::default(),
                            }
                            .into(),
                        ))
                    } else {
                        Err(error::bad_image_data(image.get_file_name()))
                    }
                }
                ty => Err(error::unsupported_param(ty)),
            }
        } else {
            Err(error::file_not_found())
        }
    }

    pub async fn upload_file_fragmented(
        &self,
        c: UploadFileFragmented,
        ob: &OneBot,
    ) -> WQRespResult {
        match c {
            UploadFileFragmented::Prepare { name, total_size } => {
                self.uploading_fragment.lock().await.cache_set(
                    format!("{}-{}", name, total_size),
                    FragmentFile {
                        total_size,
                        files: vec![],
                    },
                );
                Ok(Resps::success(
                    FileIdContent {
                        file_id: name,
                        extra: extended_map!(),
                    }
                    .into(),
                ))
            }
            UploadFileFragmented::Transfer {
                file_id,
                offset,
                size,
                data,
            } => {
                let mut file_path = std::path::PathBuf::from(crate::FILE_CACHE_DIR);
                file_path.push(format!("{}-{}", file_id, offset));
                let mut file = tokio::fs::File::create(file_path)
                    .await
                    .map_err(|e| error::file_create_error(e))?;
                file.write_all(&data)
                    .await
                    .map_err(|e| error::file_write_error(e))?;
                match self.uploading_fragment.lock().await.cache_get_mut(&file_id) {
                    Some(f) => f.files.push((offset, size)),
                    None => return Err(error::prepare_file_first()),
                }
                Ok(Resps::success(ExtendedValue::Null.into()))
            }
            UploadFileFragmented::Finish { file_id, sha256 } => {
                let sha = hex::decode(sha256).map_err(|_| error::bad_param("sha256"))?;
                let mut fragment = self
                    .uploading_fragment
                    .lock()
                    .await
                    .cache_remove(&file_id)
                    .ok_or(error::prepare_file_first())?;
                fragment.files.sort();
                let mut data = Vec::with_capacity(fragment.total_size as usize);
                let mut total_size = 0;
                for (offset, size) in fragment.files {
                    let mut file_path = std::path::PathBuf::from(crate::FILE_CACHE_DIR);
                    file_path.push(format!("{}-{}", file_id, offset));
                    let mut file = tokio::fs::File::open(file_path)
                        .await
                        .map_err(|e| error::file_open_error(e))?;
                    file.read_buf(&mut data)
                        .await
                        .map_err(|e| error::file_read_error(e))?;
                    total_size += size;
                }
                if total_size != fragment.total_size {
                    return Err(error::file_total_size_not_match());
                }
                let mut sha256 = sha2::Sha256::default();
                sha256.update(&data);
                if sha256.finalize().to_vec() != sha {
                    return Err(error::file_sha256_not_match());
                }
                self.upload_image(data, ob).await
            }
        }
    }

    pub async fn get_file_fragmented(&self, c: GetFileFragmented, _ob: &OneBot) -> WQRespResult {
        use ricq::structs::ImageInfo;
        use tokio::io::{AsyncSeekExt, SeekFrom};
        async fn to_info(
            h: &super::Handler,
            simage: impl SImage,
        ) -> Result<(ImageInfo, String), RespError> {
            let data = simage.data().await.map_err(|e| error::rq_error(e))?;
            let sha256 = {
                let mut s = sha2::Sha256::default();
                s.update(&data);
                hex::encode(&s.finalize())
            };
            let info = save_image(&data).await?;
            h.database._insert_image(&info);
            Ok((info, sha256))
        }
        match c {
            GetFileFragmented::Prepare { file_id } => {
                let (info, sha256) = match self
                    .database
                    .get_image(&hex::decode(&file_id).map_err(|_| error::bad_param("file_id"))?)
                    .ok_or(error::file_not_found())?
                {
                    Images::Friend(f) => to_info(self, f).await?,
                    Images::Group(g) => to_info(self, g).await?,
                    Images::Info(i) => {
                        let data = i.data().await.map_err(|e| error::rq_error(e))?;
                        let sha256 = {
                            let mut s = sha2::Sha256::default();
                            s.update(&data);
                            hex::encode(&s.finalize())
                        };
                        (i, sha256)
                    }
                };
                Ok(Resps::success(
                    FileFragmentedHead {
                        name: info.filename,
                        total_size: info.size as i64,
                        sha256,
                        extra: extended_map!(),
                    }
                    .into(),
                ))
            }
            GetFileFragmented::Transfer {
                file_id,
                offset,
                size,
            } => {
                let info: ImageInfo = self
                    .database
                    ._get_image(&hex::decode(&file_id).map_err(|_| error::bad_param("file_id"))?)
                    .ok_or(error::file_not_found())?;
                let mut file = tokio::fs::File::open(info.path())
                    .await
                    .map_err(|e| error::file_open_error(e))?;
                file.seek(SeekFrom::Start(offset as u64))
                    .await
                    .map_err(|e| error::file_read_error(e))?;
                let mut data = Vec::with_capacity(size as usize);
                file.read(&mut data)
                    .await
                    .map_err(|e| error::file_read_error(e))?;
                Ok(Resps::success(data.into()))
            }
        }
    }
}

pub struct FragmentFile {
    pub total_size: i64,
    pub files: Vec<(i64, i64)>,
}
