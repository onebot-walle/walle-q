use hyper::{Client, Request, Uri};
use std::collections::HashMap;
use std::str::FromStr;

pub fn http_client() -> Client<hyper::client::HttpConnector, hyper::Body> {
    Client::new()
}

pub fn https_client() -> Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>
{
    let https = hyper_tls::HttpsConnector::new();
    Client::builder().build(https)
}

pub async fn get_data_by_http(
    url: &str,
    headers: HashMap<String, String>,
) -> Result<bytes::Bytes, &'static str> {
    let uri = Uri::from_str(url).map_err(|_| "url解析失败")?;
    match uri.scheme_str() {
        Some("http") => get(&http_client(), uri, headers).await,
        Some("https") => get(&https_client(), uri, headers).await,
        _ => Err("url协议未知"),
    }
}

async fn get<C>(
    c: &Client<C>,
    uri: Uri,
    headers: HashMap<String, String>,
) -> Result<bytes::Bytes, &'static str>
where
    C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    let mut builder = Request::builder().uri(uri);
    for (k, v) in headers {
        builder = builder.header(k, v);
    }
    let req = builder.body(hyper::Body::empty()).unwrap();
    let resp = c.request(req).await.map_err(|_| "url请求失败")?;
    hyper::body::to_bytes(resp.into_body())
        .await
        .map_err(|_| "数据读取失败")
}
