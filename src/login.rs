use std::fs;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use ricq::client::{Client, DefaultConnector};
use ricq::ext::common::start_heartbeat;
use ricq::ext::reconnect::{auto_reconnect, Credential, Password};
use ricq::{LoginResponse, QRCodeState};
use ricq::{RQError, RQResult};
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, warn};
use walle_core::resp::Resp;

use crate::error::login_failed;
use crate::model::LoginResp;

#[allow(dead_code)]
const EMPTY_MD5: [u8; 16] = [
    0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8, 0x42, 0x7e,
];
const TOKEN_PATH: &str = "session.token";

pub fn token_path(uin: &str, base_path: &str) -> String {
    format!("{}/{}/{}-{}", base_path, crate::CLIENT_DIR, uin, TOKEN_PATH)
}

/// if passwords is empty use qrcode login else use password login
///
/// if login success, start client heartbeat
pub(crate) async fn console_login(
    cli: &Arc<Client>,
    uin: &str,
    password: Option<String>,
    base_path: &str,
) -> RQResult<()> {
    let token_login = token_login(cli, uin, base_path).await;
    if !token_login {
        if let (Ok(uin), Some(ref password)) = (uin.parse(), password) {
            info!(target: crate::WALLE_Q, "login with password");
            handle_login_resp(cli, cli.password_login(uin, password).await?).await?;
        } else {
            info!(target: crate::WALLE_Q, "login with qrcode");
            qrcode_login(cli).await?;
        }
    }
    after_login(cli, base_path, uin).await?;
    Ok(())
}

pub(crate) async fn token_login(cli: &Client, uin: &str, base_path: &str) -> bool {
    match fs::read(&token_path(uin, base_path)).map(|s| rmp_serde::from_slice(&s)) {
        Ok(Ok(token)) => {
            info!(
                target: crate::WALLE_Q,
                "成功读取 Token, 尝试使用 Token 登录"
            );
            match cli.token_login(token).await {
                Ok(_) => {
                    info!(target: crate::WALLE_Q, "Token 登录成功");
                    true
                }
                Err(_) => {
                    warn!(target: crate::WALLE_Q, "Token 登录失败");
                    false
                }
            }
        }
        _ => false,
    }
}

pub async fn after_login(cli: &Arc<Client>, base_path: &str, bot_id: &str) -> RQResult<()> {
    cli.register_client().await?;
    start_heartbeat(cli.clone()).await;
    let token = cli.gen_token().await;
    fs::write(
        token_path(bot_id, base_path),
        rmp_serde::to_vec(&token).unwrap(),
    )
    .unwrap();
    Ok(())
}

pub(crate) async fn start_reconnect(cli: &Arc<Client>, uin: &str, password: Option<String>) {
    let token = cli.gen_token().await;
    let credential = if let (Ok(uin), Some(ref password)) = (uin.parse(), password) {
        Credential::Password(Password {
            uin,
            password: password.to_owned(),
        })
    } else {
        Credential::Token(token)
    };
    auto_reconnect(
        cli.clone(),
        credential,
        Duration::from_secs(10),
        10,
        DefaultConnector,
    )
    .await;
}

async fn qrcode_login(cli: &Arc<Client>) -> RQResult<()> {
    let resp = cli.fetch_qrcode().await?;
    if let QRCodeState::ImageFetch(f) = resp {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("qrcode.png")
            .await
        {
            Ok(mut file) => {
                file.write_all(&f.image_data)
                    .await
                    .map_err(|e| warn!("unable to write qrcode.png file: {}", e))
                    .ok();
            }
            Err(e) => warn!("unable create qrcode.png file: {}", e),
        }
        let rended = crate::util::qrcode2str(&f.image_data);
        info!(target: crate::WALLE_Q, "扫描二维码登录:");
        println!("{}", rended);
        wait_qrcode(cli, 60, &f.sig).await
    } else {
        warn!(target: crate::WALLE_Q, "二维码获取失败");
        Err(RQError::Other("二维码获取失败".to_owned()))
    }
}

pub async fn wait_qrcode(cli: &Arc<Client>, timeout: u64, sig: &[u8]) -> RQResult<()> {
    tokio::time::timeout(std::time::Duration::from_secs(timeout), async {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            match cli.query_qrcode_result(sig).await? {
                QRCodeState::WaitingForScan => debug!("二维码待扫描"),
                QRCodeState::WaitingForConfirm => debug!("二维码待确认"),
                QRCodeState::Timeout => {
                    warn!("二维码超时");
                    break Err(RQError::Other("二维码超时".to_owned()));
                }
                QRCodeState::Confirmed(c) => {
                    info!(target: crate::WALLE_Q, "二维码已确认");
                    let resp = cli
                        .qrcode_login(&c.tmp_pwd, &c.tmp_no_pic_sig, &c.tgt_qr)
                        .await?;
                    break handle_login_resp(cli, resp).await;
                }
                QRCodeState::Canceled => {
                    warn!(target: crate::WALLE_Q, "二维码已取消");
                    warn!(target: crate::WALLE_Q, "请使用手表或 MacOS 协议扫码登录");
                    return Err(RQError::Other("二维码已取消".to_owned()));
                }
                QRCodeState::ImageFetch(_) => unreachable!(),
            }
        }
    })
    .await
    .unwrap_or_else(|_e| Err(RQError::Other("二维码等待超时".to_owned())))
}

async fn handle_login_resp(cli: &Arc<Client>, mut resp: LoginResponse) -> RQResult<()> {
    loop {
        match resp {
            LoginResponse::Success(_) => break Ok(()),
            LoginResponse::DeviceLocked(l) => {
                warn!(
                    target: crate::WALLE_Q,
                    "password login error: {}",
                    l.message.unwrap()
                );
                warn!(target: crate::WALLE_Q, "{}", l.sms_phone.unwrap());
                warn!(
                    target: crate::WALLE_Q,
                    "手机打开url, 处理完成后重启程序: {}",
                    l.verify_url.unwrap()
                );
                return Err(RQError::Other("password login failure".to_string()));
            }
            LoginResponse::DeviceLockLogin(_) => {
                resp = cli.device_lock_login().await?;
            }
            LoginResponse::AccountFrozen => {
                warn!(target: crate::WALLE_Q, "账号被冻结");
                return Err(RQError::Other("账号被冻结".to_string()));
            }
            LoginResponse::NeedCaptcha(ref captcha) => {
                if let Some(url) = &captcha.verify_url {
                    info!(target: crate::WALLE_Q, "滑块Url: {}", url);
                    info!(target: crate::WALLE_Q, "输入ticket: ");
                    let mut reader = tokio_util::codec::FramedRead::new(
                        tokio::io::stdin(),
                        tokio_util::codec::LinesCodec::new(),
                    );
                    loop {
                        if let Some(Ok(ticket)) = reader.next().await {
                            resp = cli.submit_ticket(&ticket).await?;
                            break;
                        }
                    }
                } else {
                    return Err(RQError::Other("NeedCaptcha without url".to_string()));
                }
            }
            LoginResponse::TooManySMSRequest => {
                warn!(target: crate::WALLE_Q, "短信验证码请求过于频繁");
                return Err(RQError::Other("短信验证码请求过于频繁".to_string()));
            }
            LoginResponse::UnknownStatus(s) => {
                warn!(
                    target: crate::WALLE_Q,
                    "LoginResponse UnknownStatus: {:?}", s
                );
                return Err(RQError::Other("未知状态".to_string()));
            }
        }
    }
}

pub(crate) async fn action_login(
    cli: &Arc<Client>,
    bot_id: &str,
    password: Option<String>,
    base_path: &str,
) -> RQResult<(Resp, Option<Bytes>)> {
    if token_login(cli, bot_id, base_path).await {
        return Ok((
            LoginResp {
                bot_id: bot_id.to_string(),
                url: None,
                qrcode: None,
                qrcode_str: None,
            }
            .into(),
            None,
        ));
    }
    if let (Ok(uin), Some(ref password)) = (bot_id.parse(), password) {
        info!(target: crate::WALLE_Q, "login with password");
        login_resp_to_resp(cli, cli.password_login(uin, password).await?, bot_id)
            .await
            .map(|r| (r, None))
    } else {
        info!(target: crate::WALLE_Q, "login with qrcode");
        match cli.fetch_qrcode().await? {
            QRCodeState::ImageFetch(f) => Ok((
                LoginResp {
                    bot_id: bot_id.to_string(),
                    url: None,
                    qrcode_str: Some(crate::util::qrcode2str(&f.image_data)),
                    qrcode: Some(f.image_data.to_vec().into()),
                }
                .into(),
                Some(f.sig),
            )), //todo
            _ => Ok((login_failed("二维码获取失败").into(), None)),
        }
    }
}

pub(crate) async fn login_resp_to_resp(
    cli: &Arc<Client>,
    mut resp: LoginResponse,
    bot_id: &str,
) -> RQResult<Resp> {
    if let LoginResponse::DeviceLockLogin(_) = resp {
        resp = cli.device_lock_login().await?;
    }
    Ok(match resp {
        LoginResponse::Success(_) => LoginResp {
            bot_id: bot_id.to_owned(),
            url: None,
            qrcode: None,
            qrcode_str: None,
        }
        .into(),
        LoginResponse::NeedCaptcha(n) => (
            login_failed("need_captcha"),
            LoginResp {
                bot_id: bot_id.to_owned(),
                url: n.verify_url,
                qrcode: None,
                qrcode_str: None,
            },
        )
            .into(),
        LoginResponse::DeviceLocked(l) => (
            login_failed("devicd_locked"),
            LoginResp {
                bot_id: bot_id.to_owned(),
                url: l.verify_url,
                qrcode: None,
                qrcode_str: None,
            },
        )
            .into(),
        LoginResponse::AccountFrozen => login_failed("账号被冻结").into(),
        LoginResponse::TooManySMSRequest => login_failed("短信验证码请求过于频繁").into(),
        LoginResponse::UnknownStatus(_) => login_failed("未知状态").into(),
        LoginResponse::DeviceLockLogin(_) => unreachable!(),
    })
}

#[test]
fn test_empty_md5() {
    let empty_md5 = md5::compute("".as_bytes()).0;
    assert_eq!(empty_md5, EMPTY_MD5);
}
