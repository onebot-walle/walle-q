use walle_core::prelude::{OneBot, PushToMap};

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(detail_type)]
pub struct FriendPoke {
    pub user_id: String,
    pub receiver_id: String,
}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(detail_type)]
pub struct GroupNameUpdate {
    pub group_id: String,
    pub group_name: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(detail_type)]
pub struct GroupMemberBan {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
    pub duration: i64,
}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(detail_type)]
pub struct GroupAdminSet {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(detail_type)]
pub struct GroupAdminUnset {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(sub_type)]
pub struct Join {}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(sub_type)]
pub struct Kick {}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(sub_type)]
pub struct Leave {}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(sub_type)]
pub struct Recall {}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(sub_type)]
pub struct Delete {}

#[derive(Debug, Clone, PushToMap, OneBot)]
#[event(sub_type)]
pub struct Disband {}
