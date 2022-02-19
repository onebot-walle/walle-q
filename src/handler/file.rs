use std::path::PathBuf;

use tokio::fs;
use walle_core::{action::UploadFileContent, impls::OneBot, resp::FileIdContent, Resps};

const FILE_PATH: &str = "./data/file/";

impl super::Handler {
    pub async fn upload_file(&self, c: UploadFileContent, _ob: &OneBot) -> Resps {
        match c.r#type.as_str() {
            "url" if let Some(url) = c.url => {
                todo!()
            }
            "path" if let Some(path) = c.path => {
                todo!()
            }
            "data" if let Some(data) = c.data => {
                let file_id = walle_core::new_uuid();
                let file_path = PathBuf::from(format!("{}{}", FILE_PATH, file_id));
                fs::write(&file_path, data).await.unwrap(); //todo
                Resps::success(
                    FileIdContent {
                        file_id: file_id.to_string(),
                    }
                    .into(),
                )
            }
            _ => Resps::bad_param(),
        }
    }
}
