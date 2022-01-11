use std::time::Duration;

use rs_qq::client::income::decoder::wtlogin::{LoginResponse, QRCodeState};
use rs_qq::client::Client;
use rs_qq::error::{RQError, RQResult};
use std::sync::Arc;
use tracing::{debug, info, warn};

const EMPTY_MD5: [u8; 16] = [
    0xd4, 0x1d, 0x8c, 0xd9, 0x8f, 0x00, 0xb2, 0x04, 0xe9, 0x80, 0x09, 0x98, 0xec, 0xf8, 0x42, 0x7e,
];

/// if passwords is empty use qrcode login else use password login
///
/// if login success, start client heartbeat
pub(crate) async fn login(cli: &Arc<Client>) -> RQResult<()> {
    if &cli.password_md5[..] == &EMPTY_MD5 {
        info!("login with qrcode");
        qrcode_login(cli).await?;
    } else {
        info!("login with password");
        password_login(cli).await?;
    }
    cli.register_client().await?;
    let ncli = cli.clone();
    tokio::spawn(async move {
        ncli.do_heartbeat().await;
    });
    cli.reload_friend_list().await?;
    cli.reload_group_list().await?;
    Ok(())
}

async fn qrcode_login(cli: &Client) -> RQResult<()> {
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

async fn password_login(cli: &Client) -> RQResult<()> {
    handle_login_resp(cli, cli.password_login().await?).await
}

async fn handle_login_resp(cli: &Client, mut resp: LoginResponse) -> RQResult<()> {
    loop {
        match resp {
            LoginResponse::Success => break Ok(()),
            LoginResponse::SMSOrVerifyNeededError {
                verify_url,
                sms_phone,
                error_message,
            } => {
                warn!("password login error: {}", error_message);
                warn!("{}", sms_phone);
                warn!("手机打开url，处理完成后重启程序: {}", verify_url);
                return Err(RQError::Other("password login failure".to_string()));
            }
            LoginResponse::NeedDeviceLockLogin => {
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
