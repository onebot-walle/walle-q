use walle_core::prelude::{PushToValueMap, ToEvent};

#[derive(Debug, Clone, PushToValueMap, ToEvent)]
#[event(detail_type)]
pub struct NewFriend {
    pub request_id: i64,
    pub user_id: String,
    pub user_name: String,
    pub message: String,
}

#[derive(Debug, Clone, PushToValueMap, ToEvent)]
#[event(detail_type)]
pub struct JoinGroup {
    pub request_id: i64,
    pub user_id: String,
    pub user_name: String,
    pub group_id: String,
    pub group_name: String,
    pub message: String,
    pub suspicious: bool,
    pub invitor_id: Option<String>,
    pub invitor_name: Option<String>,
}

#[derive(Debug, Clone, PushToValueMap, ToEvent)]
#[event(detail_type)]
pub struct GroupInvite {
    pub request_id: i64,
    pub group_id: String,
    pub group_name: String,
    pub invitor_id: String,
    pub invitor_name: String,
}
