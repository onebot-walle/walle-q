use std::{collections::HashMap, path::PathBuf};

use crate::database::{save_image, Database, SImage};

use super::ResultFlatten;
use tokio::{fs::File, io::AsyncReadExt};
use walle_core::{action::UploadFileContent, impls::OneBot, Resps};

impl super::Handler {
    async fn get_file_data(c: UploadFileContent) -> Result<Vec<u8>, Resps> {
        match c.r#type.as_str() {
            "url" if let Some(url) = c.url => {
                get_date_by_url(&url, c.headers.unwrap_or_default()).await
            }
            "path" if let Some(path) = c.path => {
                let input_path = PathBuf::from(path);
                let mut file =  File::open(&input_path).await.map_err(|_| {
                    Resps::empty_fail(10003,  "文件打开失败".to_string())
                })?;
                let mut data = Vec::new();
                file.read_to_end(&mut data).await.map_err(|_| {
                    Resps::empty_fail(10003,  "文件读取失败".to_string())
                })?;
                Ok(data.into())
            }
            "data" if let Some(data) = c.data => Ok(data),
            _ => Err(Resps::bad_param()),
        }
    }

    pub async fn upload_file(&self, c: UploadFileContent, ob: &OneBot) -> Resps {
        let fut = || async {
            let file_type = c
                .extra
                .get("file_type")
                .ok_or(Resps::bad_param())?
                .clone()
                .downcast_str()
                .map_err(|_| Resps::bad_param())?;
            let data = Self::get_file_data(c).await?;
            match file_type.as_str() {
                "image" => self.upload_image(data, ob).await,
                _ => Err(Resps::bad_param()),
            }
        };
        fut().await.flatten()
    }

    pub async fn upload_image(&self, data: Vec<u8>, _ob: &OneBot) -> Result<Resps, Resps> {
        let info = save_image(&data).await.map_err(|m| {
            Resps::empty_fail(32000, m.to_string())
            //todo
        })?;
        crate::WQDB._insert_image(&info);
        Ok(Resps::success(info.as_file_id_content().into()))
    }
}

async fn get_date_by_url(url: &str, headers: HashMap<String, String>) -> Result<Vec<u8>, Resps> {
    uri_reader::uget_with_headers(url, headers)
        .await
        .map_err(|m| Resps::empty_fail(10003, m.to_string()))
}
