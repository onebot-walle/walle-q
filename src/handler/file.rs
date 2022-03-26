use std::{collections::HashMap, path::PathBuf};

use crate::database::{save_image, Database, SImage};
use crate::error::{WQError, WQResult};

use tokio::{fs::File, io::AsyncReadExt};
use walle_core::{
    action::{GetFileContent, UploadFileContent},
    impls::OneBot,
    Resps,
};

impl super::Handler {
    async fn get_file_data(c: UploadFileContent) -> WQResult<Vec<u8>> {
        match c.r#type.as_str() {
            "url" if let Some(url) = c.url => {
                get_date_by_url(&url, c.headers.unwrap_or_default()).await
            }
            "path" if let Some(path) = c.path => {
                let input_path = PathBuf::from(path);
                let mut file =  File::open(&input_path).await.map_err(|e| {
                    WQError::file_open_error(e)
                })?;
                let mut data = Vec::new();
                file.read_to_end(&mut data).await.map_err(|e| {
                    WQError::file_read_error(e)
                })?;
                Ok(data.into())
            }
            "data" if let Some(data) = c.data => Ok(data),
            _ => Err(WQError::bad_param("type")),
        }
    }

    pub async fn upload_file(&self, c: UploadFileContent, ob: &OneBot) -> WQResult<Resps> {
        let file_type = c
            .extra
            .get("file_type")
            .ok_or(WQError::bad_param("file_type"))?
            .clone()
            .downcast_str()
            .map_err(|_| WQError::bad_param("file_type"))?;
        let data = Self::get_file_data(c).await?;
        match file_type.as_str() {
            "image" => self.upload_image(data, ob).await,
            _ => Err(WQError::bad_param("file_type")),
        }
    }

    pub async fn upload_image(&self, data: Vec<u8>, _ob: &OneBot) -> WQResult<Resps> {
        let info = save_image(&data).await?;
        crate::WQDB._insert_image(&info);
        Ok(Resps::success(info.as_file_id_content().into()))
    }

    pub async fn get_file(&self, c: GetFileContent, ob: &OneBot) -> WQResult<Resps> {
        let file_type = c
            .extra
            .get("file_type")
            .ok_or(WQError::bad_param("file_type"))?
            .clone()
            .downcast_str()
            .map_err(|_| WQError::bad_param("file_type"))?;
        match file_type.as_str() {
            "image" => self.get_image(&c, ob).await,
            _ => Err(WQError::bad_param("file_type")),
        }
    }

    pub async fn get_image(&self, c: &GetFileContent, _ob: &OneBot) -> WQResult<Resps> {
        if let Some(image) = hex::decode(&c.file_id)
            .ok()
            .and_then(|id| crate::WQDB.get_image(&id))
        {
            match c.r#type.as_str() {
                "url" => {
                    if let Some(url) = image.get_url() {
                        Ok(Resps::success(
                            UploadFileContent {
                                r#type: "url".to_string(),
                                name: image.get_file_name().to_string(),
                                url: Some(url),
                                ..Default::default()
                            }
                            .into(),
                        ))
                    } else {
                        Err(WQError::image_url(image.get_file_name()))
                    }
                }
                "path" => {
                    if image.path().exists() {
                        Ok(Resps::success(
                            UploadFileContent {
                                r#type: "path".to_string(),
                                name: image.get_file_name().to_string(),
                                path: Some(image.path().to_str().unwrap().to_string()),
                                ..Default::default()
                            }
                            .into(),
                        ))
                    } else {
                        Err(WQError::image_path(image.get_file_name()))
                    }
                }
                "data" => {
                    if let Ok(data) = image.data().await {
                        Ok(Resps::success(
                            UploadFileContent {
                                r#type: "data".to_string(),
                                name: image.get_file_name().to_string(),
                                data: Some(data),
                                ..Default::default()
                            }
                            .into(),
                        ))
                    } else {
                        Err(WQError::image_data(image.get_file_name()))
                    }
                }
                _ => Err(WQError::bad_param("type")),
            }
        } else {
            Err(WQError::image_unuploaded())
        }
    }
}

async fn get_date_by_url(url: &str, headers: HashMap<String, String>) -> WQResult<Vec<u8>> {
    uri_reader::uget_with_headers(url, headers)
        .await
        .map_err(|m| WQError::net_download_fail(m))
}
