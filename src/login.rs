use std::fs;
use std::time::Duration;

use rs_qq::client::Client;
use rs_qq::{LoginResponse, QRCodeState};
use rs_qq::{RQError, RQResult};
use std::sync::Arc;
use tracing::{debug, info, warn};

#[allow(dead_code)]
const EMPTY_MD5: [u8; 16] = [
    0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8, 0x42, 0x7e,
];
const TOKEN_PATH: &str = "session.token";

/// if passwords is empty use qrcode login else use password login
///
/// if login success, start client heartbeat
pub(crate) async fn login(cli: &Arc<Client>, config: &crate::config::QQConfig) -> RQResult<()> {
    let token_login: bool = match fs::read(TOKEN_PATH) {
        Ok(token) => {
            info!("成功读取 Token, 尝试使用 Token 登录");
            match cli.token_login(token.as_slice()).await {
                Ok(_) => {
                    info!("Token 登录成功");
                    true
                }
                Err(_) => {
                    warn!("Token 登录失败");
                    false
                }
            }
        }
        Err(_) => false,
    };
    if !token_login {
        if let (Some(uin), Some(password)) = (config.uin, &config.password) {
            info!("login with password");
            handle_login_resp(cli, cli.password_login(uin as i64, password).await?).await?;
        } else {
            info!("login with qrcode");
            qrcode_login(cli).await?;
        }
        let token = cli.gen_token().await;
        fs::write(TOKEN_PATH, token).unwrap();
        cli.register_client().await?;
    }
    let ncli = cli.clone();
    tokio::spawn(async move {
        ncli.do_heartbeat().await;
    });
    cli.reload_friends().await?;
    cli.reload_groups().await
}

async fn qrcode_login(cli: &Arc<Client>) -> RQResult<()> {
    let resp = cli.fetch_qrcode().await?;
    if let QRCodeState::QRCodeImageFetch { image_data, sig } = resp {
        tokio::fs::write("qrcode.png", &image_data)
            .await
            .map_err(|_| RQError::Other("fail to write qrcode.png file".to_owned()))?;
        info!("请打开 qrcode.png 文件扫描二维码登录");
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            match cli.query_qrcode_result(&sig).await? {
                QRCodeState::QRCodeWaitingForScan => debug!("二维码待扫描"),
                QRCodeState::QRCodeWaitingForConfirm => debug!("二维码待确认"),
                QRCodeState::QRCodeTimeout => {
                    warn!("二维码超时");
                    break Err(RQError::Other("二维码超时".to_owned()));
                }
                QRCodeState::QRCodeConfirmed {
                    tmp_pwd,
                    tmp_no_pic_sig,
                    tgt_qr,
                    ..
                } => {
                    info!("二维码已确认");
                    let resp = cli.qrcode_login(&tmp_pwd, &tmp_no_pic_sig, &tgt_qr).await?;
                    break handle_login_resp(cli, resp).await;
                }
                _ => todo!(),
            }
        }
    } else {
        warn!("二维码获取失败");
        Err(RQError::Other("二维码获取失败".to_owned()))
    }
}

async fn handle_login_resp(cli: &Arc<Client>, mut resp: LoginResponse) -> RQResult<()> {
    loop {
        match resp {
            LoginResponse::Success { .. } => break Ok(()),
            LoginResponse::DeviceLocked {
                verify_url,
                sms_phone,
                message,
                ..
            } => {
                warn!("password login error: {}", message.unwrap());
                warn!("{}", sms_phone.unwrap());
                warn!("手机打开url，处理完成后重启程序: {}", verify_url.unwrap());
                return Err(RQError::Other("password login failure".to_string()));
            }
            LoginResponse::DeviceLockLogin { .. } => {
                resp = cli.device_lock_login().await?;
            }
            _ => unimplemented!(),
        }
    }
}

#[test]
fn test_empty_md5() {
    let empty_md5 = md5::compute("".as_bytes()).0;
    assert_eq!(empty_md5, EMPTY_MD5);
}
