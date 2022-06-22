use serde::{Deserialize, Serialize};
use walle_core::{ExtendedMap, ExtendedMapExt, SelfId, StandardAction};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetNewFriend {
    pub user_id: String,
    pub request_id: i64,
    pub accept: bool,
    pub self_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteFriend {
    pub user_id: String,
    pub self_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetJoinGroup {
    pub request_id: i64,
    pub user_id: String,
    pub group_id: String,
    pub accept: bool,
    pub block: Option<bool>,
    pub message: Option<String>,
    pub self_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetGroupInvited {
    pub request_id: i64,
    pub group_id: String,
    pub accept: bool,
    pub self_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "params", rename_all = "snake_case")]
pub enum WQExtraAction {
    SetNewFriend(SetNewFriend),
    DeleteFriend(DeleteFriend),
    GetNewFriendRequests(ExtendedMap),
    SetJoinGroup(SetJoinGroup),
    GetJoinGroupRequests(ExtendedMap),
    SetGroupInvited(SetGroupInvited),
    GetGroupInviteds(ExtendedMap),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum WQAction {
    Standard(StandardAction),
    Extra(WQExtraAction),
}

impl SelfId for WQAction {
    fn self_id(&self) -> String {
        match self {
            Self::Standard(s) => s.self_id(),
            Self::Extra(e) => e.self_id(),
        }
    }
}

fn default_str(i: &Option<String>) -> String {
    match i {
        Some(s) => s.clone(),
        None => String::default(),
    }
}

impl SelfId for WQExtraAction {
    fn self_id(&self) -> String {
        match self {
            Self::SetNewFriend(s) => default_str(&s.self_id),
            Self::DeleteFriend(d) => default_str(&d.self_id),
            Self::GetNewFriendRequests(e) => e.try_get("self_id").unwrap_or_default(),
            Self::SetJoinGroup(s) => default_str(&s.self_id),
            Self::GetJoinGroupRequests(e) => e.try_get("self_id").unwrap_or_default(),
            Self::SetGroupInvited(s) => default_str(&s.self_id),
            Self::GetGroupInviteds(e) => e.try_get("self_id").unwrap_or_default(),
        }
    }
}
