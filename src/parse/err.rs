use rs_qq::RQError;
use walle_core::Resps;

use thiserror::Error;

pub type WQResult<T> = Result<T, WQError>;

#[derive(Error, Debug)]
pub enum WQError {
    #[error("{0}")]
    RQ(#[from] RQError),
    #[error("{0}")]
    WQ(&'static str),
}

pub(crate) fn rqerror_to_resps(error: RQError) -> Resps {
    //todo
    Resps::empty_fail(34001, error.to_string())
}

macro_rules! wqerror_codes {
    ($($t: ident => $msg: expr, $code: expr),*) => {
        impl From<WQError> for Resps {
            fn from(e: WQError) -> Self {
                match e {
                    WQError::RQ(e) => rqerror_to_resps(e),
                    WQError::WQ(e) => match e {
                        $(
                        $msg => Resps::empty_fail($code, $msg.to_string()),
                        )*
                        _ => Resps::empty_fail(40000, e.to_string()),
                    },
                }
            }
        }
        impl WQError {
            $(
            pub fn $t() -> Self {
                WQError::WQ($msg)
            }
            )*
        }
    };
}

wqerror_codes!(
    image_unuploaded => "图片未上传", 32001,
    image_not_exist => "图片不存在", 34001);
