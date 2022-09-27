use walle_core::{prelude::PushToValueMap, util::OneBotBytes};

#[derive(PushToValueMap)]
pub struct LoginResp {
    pub user_id: String,
    pub url: Option<String>,
    pub qrcode: Option<OneBotBytes>,
}
