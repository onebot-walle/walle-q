use std::fs;
use std::sync::Arc;
use std::time::Duration;

use ricq::client::Client;
use ricq::ext::common::after_login;
use ricq::ext::reconnect::{auto_reconnect, Credential, DefaultConnector, Password};
use ricq::{LoginResponse, QRCodeState};
use ricq::{RQError, RQResult};
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
    let token_login: bool = match fs::read(TOKEN_PATH).map(|s| rmp_serde::from_slice(&s)) {
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
    };
    if !token_login {
        if let (Some(uin), Some(password)) = (config.uin, &config.password) {
            info!(target: crate::WALLE_Q, "login with password");
            handle_login_resp(cli, cli.password_login(uin as i64, password).await?).await?;
        } else {
            info!(target: crate::WALLE_Q, "login with qrcode");
            qrcode_login(cli).await?;
        }
        let token = cli.gen_token().await;
        fs::write(TOKEN_PATH, rmp_serde::to_vec(&token).unwrap()).unwrap();
        cli.register_client().await?;
    }
    after_login(cli).await;
    Ok(())
}

pub(crate) async fn start_reconnect(cli: &Arc<Client>, config: &crate::config::QQConfig) {
    let token = cli.gen_token().await;
    let credential = if let (Some(uin), Some(password)) = (config.uin, &config.password) {
        Credential::Password(Password {
            uin: uin as i64,
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
        let mut image = rqrr::PreparedImage::prepare(
            image::load_from_memory(&f.image_data).unwrap().into_luma8(),
        );
        let grids = image.detect_grids();
        let (_, content) = grids[0].decode().unwrap();
        let code = qrcode::QrCode::new(&content).unwrap();
        let rended = code.render::<qrcode::render::unicode::Dense1x2>().build();
        info!(target: crate::WALLE_Q, "扫描二维码登录:");
        println!("{}", rended);
        // tokio::fs::write("qrcode.png", &f.image_data)
        //     .await
        //     .map_err(|_| RQError::Other("fail to write qrcode.png file".to_owned()))?;
        // info!(
        //     target: crate::WALLE_Q,
        //     "请打开 qrcode.png 文件扫描二维码登录"
        // );
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            match cli.query_qrcode_result(&f.sig).await? {
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
                _ => todo!(),
            }
        }
    } else {
        warn!(target: crate::WALLE_Q, "二维码获取失败");
        Err(RQError::Other("二维码获取失败".to_owned()))
    }
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
                    "手机打开url，处理完成后重启程序: {}",
                    l.verify_url.unwrap()
                );
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
