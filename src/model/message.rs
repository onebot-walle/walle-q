use walle_core::prelude::{OneBot, PushToMap};

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(detail_type)]
pub struct GroupTemp {
    pub group_id: String,
}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(platform = "qq")]
pub struct Names {
    pub group_name: String,
    pub user_name: String,
}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(platform = "qq")]
pub struct UserName {
    pub user_name: String,
}
