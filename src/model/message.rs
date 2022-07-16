use walle_core::prelude::{OneBot, PushToValueMap};

#[derive(Debug, Clone, PushToValueMap, OneBot)]
#[event(detail_type)]
pub struct GroupTemp {
    pub group_id: String,
}

#[derive(Debug, Clone, PushToValueMap, OneBot)]
#[event(platform = "qq")]
pub struct Names {
    pub group_name: String,
    pub user_name: String,
}

#[derive(Debug, Clone, PushToValueMap, OneBot)]
#[event(platform = "qq")]
pub struct UserName {
    pub user_name: String,
}
