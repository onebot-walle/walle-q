use ricq::RQError;
use walle_core::Resps;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WQError {
    #[error("{0}")]
    RQ(#[from] RQError),
    #[error("{0}")]
    Static(&'static str),
    #[error("{0}:{1}")]
    Str(&'static str, String),
}

pub type WQResult<T> = Result<T, WQError>;

pub(crate) fn rqerror_to_resps(error: RQError) -> Resps {
    Resps::empty_fail(34001, error.to_string())
}

macro_rules! wqerror_codes {
    ($($t: ident => $msg: expr, $code: expr),*;$($t1: ident => $msg1: expr, $code1: expr),*) => {
        impl From<WQError> for Resps {
            fn from(e: WQError) -> Self {
                match e {
                    WQError::RQ(e) => rqerror_to_resps(e),
                    WQError::Static(e) => match e {
                        $(
                        $msg => Resps::empty_fail($code, $msg.to_string()),
                        )*
                        _ => Resps::empty_fail(20002, e.to_string()),
                    },
                    WQError::Str(s, _) => match s {
                        $(
                        $msg1 => Resps::empty_fail($code1, e.to_string()),
                        )*
                        _ => Resps::empty_fail(20002, e.to_string()),
                    },
                }
            }
        }
        impl WQError {
            $(
            pub fn $t() -> Self {
                WQError::Static($msg)
            }
            )*
            $(
            pub fn $t1<T: ToString>(m: T) -> Self
            {
                WQError::Str($msg1, m.to_string())
            }
            )*
        }
    };
}

wqerror_codes!(
    unsupported_action => "不支持的动作请求", 10002,
    empty_message => "消息为空", 10003,
    image_unuploaded => "图片未上传", 32001,
    message_not_exist => "消息不存在", 35001,
    friend_not_exist => "好友不存在", 35002,
    group_not_exist => "群不存在", 35003,
    group_member_not_exist => "群成员不存在", 35004,
    image_info_decode_error => "图片解码错误", 41001;
    bad_param => "参数错误", 10003,
    unsupported_param => "不支持的参数", 10004,
    file_open_error => "文件打开失败", 32001,
    file_read_error => "文件读取失败", 32002,
    file_create_error => "文件创建失败", 32003,
    file_write_error => "文件写入失败", 32004,
    net_download_fail => "网络下载失败", 33001,
    image_url => "图片URL错误", 41002,
    image_path => "图片路径错误", 41003,
    image_data => "图片内容错误", 41004
);
