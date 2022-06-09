use serde::{Deserialize, Serialize};
use walle_core::{ExtendedValue, StandardAction};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetNewFriend {
    pub user_id: String,
    pub request_id: i64,
    pub accept: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteFriend {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetJoinGroup {
    pub request_id: i64,
    pub user_id: String,
    pub group_id: String,
    pub accept: bool,
    pub block: Option<bool>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetGroupInvited {
    pub request_id: i64,
    pub group_id: String,
    pub accept: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "params", rename_all = "snake_case")]
pub enum WQExtraAction {
    SetNewFriend(SetNewFriend),
    DeleteFriend(DeleteFriend),
    GetNewFriendRequests(ExtendedValue),
    SetJoinGroup(SetJoinGroup),
    GetJoinGroupRequests(ExtendedValue),
    SetGroupInvited(SetGroupInvited),
    GetGroupInviteds(ExtendedValue),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum WQAction {
    Standard(StandardAction),
    Extra(WQExtraAction),
}
