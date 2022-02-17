use rs_qq::RQError;
use walle_core::Resps;

pub(crate) fn error_to_resps(error: RQError) -> Resps {
    //todo
    Resps::empty_fail(34001, error.to_string())
}
