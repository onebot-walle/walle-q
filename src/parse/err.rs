use rs_qq::RQError;
use walle_core::Resps;

pub const IMAGE_NOT_EXIST: &str = "图片不存在";

pub(crate) fn error_to_resps(error: RQError) -> Resps {
    match error {
        RQError::Other(s) => Resps::empty_fail(error_code(&s), s),
        error => Resps::empty_fail(34001, error.to_string()),
    }
}

fn error_code(s: &str) -> i64 {
    match s {
        IMAGE_NOT_EXIST => 31001,
        _ => 40000,
    }
}
