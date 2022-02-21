use std::{collections::HashMap, path::PathBuf, str::FromStr};

use super::ResultFlatten;
use hyper::{Client, Request, Uri};
use tokio::{fs::File, io::AsyncReadExt};
use walle_core::{action::UploadFileContent, impls::OneBot, Resps};

impl super::Handler {
    async fn get_file_data(c: UploadFileContent) -> Result<bytes::Bytes, Resps> {
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
            "data" if let Some(data) = c.data => Ok(bytes::Bytes::copy_from_slice(&data)),
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

    pub async fn upload_image(&self, _data: bytes::Bytes, _ob: &OneBot) -> Result<Resps, Resps> {
        todo!()
    }
}

async fn get_date_by_url(
    url: &str,
    headers: HashMap<String, String>,
) -> Result<bytes::Bytes, Resps> {
    async fn _get<C>(
        c: &Client<C>,
        uri: Uri,
        headers: HashMap<String, String>,
    ) -> Result<bytes::Bytes, Resps>
    where
        C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        let mut builder = Request::builder().uri(uri);
        for (k, v) in headers {
            builder = builder.header(k, v);
        }
        let req = builder.body(hyper::Body::empty()).unwrap();
        let resp = c
            .request(req)
            .await
            .map_err(|_| Resps::empty_fail(10003, "url请求失败".to_string()))?;
        hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(|_| Resps::empty_fail(10003, "数据读取失败".to_string()))
    }

    let uri =
        Uri::from_str(url).map_err(|_| Resps::empty_fail(10003, "url格式错误".to_string()))?;
    match uri.scheme().map(|s| s.as_str()) {
        Some("http") => {
            let client = hyper::Client::new();
            _get(&client, uri, headers).await
        }
        Some("https") => {
            let https = hyper_tls::HttpsConnector::new();
            let client = hyper::Client::builder().build::<_, hyper::Body>(https);
            _get(&client, uri, headers).await
        }
        Some(x) => Err(Resps::empty_fail(10003, format!("不支持的协议{}", x))),
        None => Err(Resps::empty_fail(10003, "url格式错误".to_string())),
    }
}
