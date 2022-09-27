use walle_core::action::*;
use walle_core::prelude::{PushToValueMap, TryFromAction, TryFromValue};
use walle_core::util::OneBotBytes;

#[derive(Debug, Clone, TryFromValue)]
pub struct GetMessage {
    pub message_id: String,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct KickGroupMember {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct BanGroupMember {
    pub group_id: String,
    pub user_id: String,
    pub duration: u32,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct UnbanGroupMember {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct SetGroupAdmin {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct UnsetGroupAdmin {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct SetNewFriend {
    pub user_id: String,
    pub request_id: i64,
    pub accept: bool,
    pub self_id: Option<String>,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct DeleteFriend {
    pub user_id: String,
    pub self_id: Option<String>,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct SetJoinGroup {
    pub request_id: i64,
    pub user_id: String,
    pub group_id: String,
    pub accept: bool,
    pub block: Option<bool>,
    pub message: Option<String>,
    pub self_id: Option<String>,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct SetGroupInvited {
    pub request_id: i64,
    pub group_id: String,
    pub accept: bool,
    pub self_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromValue, PushToValueMap)]
pub struct WQUploadFile {
    pub ty: String,
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<OneBotBytes>,
    pub sha256: Option<String>,
    pub file_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromValue)]
pub struct WQGetFile {
    pub file_id: String,
    pub ty: String,
    pub file_type: Option<String>,
}

#[derive(Debug, Clone, TryFromAction)]
pub struct LoginClient {
    pub uin: String,
    pub password: Option<String>,
    pub password_md5: Option<String>,
    pub protcol: u8,
}

#[derive(Debug, TryFromAction)]
pub struct SubmitTicket {
    pub user_id: String,
    pub ticket: String,
}

#[derive(Debug, Clone, TryFromAction)]
pub enum WQAction {
    GetLatestEvents(GetLatestEvents),
    GetSupportedActions {},
    GetStatus {},
    GetVersion {},

    SendMessage(SendMessage),
    DeleteMessage(DeleteMessage),
    GetMessage(GetMessage),

    GetSelfInfo {},
    GetUserInfo(GetUserInfo),
    GetFriendList {},

    GetGroupInfo(GetGroupInfo),
    GetGroupList {},
    GetGroupMemberInfo(GetGroupMemberInfo),
    GetGroupMemberList(GetGroupMemberList),
    SetGroupName(SetGroupName),
    LeaveGroup(LeaveGroup),
    UploadFile(WQUploadFile),
    UploadFileFragmented(UploadFileFragmented),
    GetFile(WQGetFile),
    GetFileFragmented(GetFileFragmented),

    KickGroupMember(KickGroupMember),
    BanGroupMember(BanGroupMember),
    UnbanGroupMember(UnbanGroupMember),
    SetGroupAdmin(SetGroupAdmin),
    UnsetGroupAdmin(UnsetGroupAdmin),

    SetNewFriend(SetNewFriend),
    DeleteFriend(DeleteFriend),
    GetNewFriendRequests {},

    SetJoinGroup(SetJoinGroup),
    GetJoinGroupRequests {},
    SetGroupInvited(SetGroupInvited),
    GetGroupInviteds {},
}
