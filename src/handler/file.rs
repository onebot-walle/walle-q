use std::path::PathBuf;

use tokio::{fs::File, io::AsyncReadExt};
use walle_core::action::{GetFile, UploadFile};
use walle_core::resp::RespError;
use walle_core::{ExtendedMap, Resps};

use crate::database::{save_image, Database, SImage};
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
        self.2._insert_image(&info);
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
            .and_then(|id| self.2.get_image(&id))
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
            Err(error::image_unuploaded())
        }
    }
}
