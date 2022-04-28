use crate::database::{Database, SGroupMessage, SMessage, SPrivateMessage, WQDatabase};
use crate::error::{WQError, WQResult};
use crate::parse::MsgChainBuilder;
use async_trait::async_trait;
use cached::SizedCache;
use std::sync::Arc;
use tokio::sync::Mutex;
use walle_core::action::{
    DeleteMessage, GetGroupInfo, GetGroupMemberInfo, GetGroupMemberList, GetLatestEvents,
    GetUserInfo, KickGroupMember, LeaveGroup, SendMessage, SetGroupName,
};
use walle_core::ExtendedMap;
use walle_core::{
    impls::OneBot,
    resp::{
        GroupInfoContent, SendMessageRespContent, StatusContent, UserInfoContent, VersionContent,
    },
    ActionHandler, ColoredAlt, RespContent, Resps, StandardAction, StandardEvent,
};

mod file;
pub(crate) mod v11;

pub(crate) struct Handler(
    pub(crate) Arc<rs_qq::Client>,
    pub(crate) Arc<Mutex<SizedCache<String, StandardEvent>>>,
    pub(crate) Arc<WQDatabase>,
);

#[async_trait]
impl ActionHandler<StandardAction, Resps, OneBot> for Handler {
    async fn handle(&self, action: StandardAction, ob: &OneBot) -> Resps {
        if let Some(alt) = action.colored_alt() {
            tracing::info!(target: crate::WALLE_Q, "{}", alt);
        }
        match match action {
            StandardAction::GetLatestEvents(c) => self.get_latest_events(c, ob).await,
            StandardAction::GetSupportedActions(_) => Self::get_supported_actions(),
            StandardAction::GetStatus(_) => self.get_status(),
            StandardAction::GetVersion(_) => Self::get_version(),

            StandardAction::SendMessage(c) => self.send_message(c, ob).await,
            StandardAction::DeleteMessage(c) => self.delete_message(c, ob).await,

            StandardAction::GetSelfInfo(_) => self.get_self_info().await,
            StandardAction::GetUserInfo(c) => self.get_user_info(c, ob).await,
            StandardAction::GetFriendList(_) => self.get_friend_list().await,

            StandardAction::GetGroupInfo(c) => self.get_group_info(c, ob).await,
            StandardAction::GetGroupList(_) => self.get_group_list().await,
            StandardAction::GetGroupMemberInfo(c) => self.get_group_member_info(c).await,
            StandardAction::GetGroupMemberList(c) => self.get_group_member_list(c).await,
            StandardAction::SetGroupName(c) => self.set_group_name(c, ob).await,
            StandardAction::LeaveGroup(c) => self.leave_group(c, ob).await,
            StandardAction::KickGroupMember(c) => self.kick_group_member(c, ob).await,
            StandardAction::BanGroupMember(c) => {
                self.ban_group_member(c.group_id, c.user_id, c.extra, ob, false)
                    .await
            }
            StandardAction::UnbanGroupMember(c) => {
                self.ban_group_member(c.group_id, c.user_id, c.extra, ob, true)
                    .await
            }
            StandardAction::SetGroupAdmin(c) => {
                self.set_group_admin(c.group_id, c.user_id, ob, false).await
            }
            StandardAction::UnsetGroupAdmin(c) => {
                self.set_group_admin(c.group_id, c.user_id, ob, true).await
            }

            StandardAction::UploadFile(c) => self.upload_file(c, ob).await,
            StandardAction::UploadFileFragmented(_c) => Err(WQError::unsupported_action()),
            StandardAction::GetFile(c) => self.get_file(c, ob).await,
            StandardAction::GetFileFragmented(_c) => Err(WQError::unsupported_action()),
        } {
            Ok(resps) => resps,
            Err(e) => e.into(),
        }
    }
}

impl Handler {
    async fn get_latest_events(&self, c: GetLatestEvents, _ob: &OneBot) -> WQResult<Resps> {
        let get = || async {
            self.1
                .lock()
                .await
                .value_order()
                .take(if c.limit <= 0 { 10 } else { c.limit as usize })
                .cloned()
                .collect::<Vec<_>>()
        };
        let mut events = get().await;
        if events.is_empty() && c.timeout != 0 {
            tokio::time::sleep(std::time::Duration::from_secs(c.timeout as u64)).await;
            events = get().await;
        }
        Ok(Resps::success(events.into()))
    }
    fn get_supported_actions() -> WQResult<Resps> {
        Ok(Resps::success(RespContent::SupportActions(vec![
            "get_latest_events".into(),
            "get_supported_actions".into(),
            "get_status".into(),
            "get_version".into(),
            "send_message".into(),
            "delete_message".into(),
            "get_self_info".into(),
            "get_user_info".into(),
            "get_friend_list".into(),
            "get_group_info".into(),
            "get_group_list".into(),
            "get_group_member_list".into(),
            "get_group_member_info".into(),
            "set_group_name".into(),
            "kick_group_member".into(),
            "ban_group_member".into(),
            "unban_group_member".into(),
            "set_group_admin".into(),
            "unset_group_admin".into(),
            "upload_file".into(),
            "get_file".into(),
        ])))
    }
    fn get_version() -> WQResult<Resps> {
        Ok(Resps::success(
            VersionContent {
                r#impl: crate::WALLE_Q.to_string(),
                platform: "qq".to_string(),
                version: crate::VERSION.to_string(),
                onebot_version: OneBot::onebot_version().to_string(),
            }
            .into(),
        ))
    }
    fn get_status(&self) -> WQResult<Resps> {
        Ok(Resps::success(
            StatusContent {
                good: true,
                online: self.0.online.load(std::sync::atomic::Ordering::Relaxed),
            }
            .into(),
        ))
    }

    async fn send_message(&self, c: SendMessage, _ob: &OneBot) -> WQResult<Resps> {
        if &c.detail_type == "group" {
            let group_id = c.group_id.ok_or(WQError::bad_param("group_id"))?;
            let group_code = group_id
                .parse()
                .map_err(|_| WQError::bad_param("group_id"))?;
            if let Some(chain) =
                MsgChainBuilder::group_chain_builder(&self.0, group_code, c.message.clone())
                    .build(&self.2)
                    .await
            {
                let receipt = self
                    .0
                    .send_group_message(group_code, chain)
                    .await
                    .map_err(WQError::RQ)?;
                let message_id = receipt.seqs[0].to_string();
                let respc = SendMessageRespContent {
                    message_id,
                    time: receipt.time as f64,
                };
                let s_group =
                    SGroupMessage::receipt(receipt, group_code, self.0.uin().await, c.message);
                self.2.insert_group_message(&s_group);
                Ok(Resps::success(respc.into()))
            } else {
                Err(WQError::empty_message())
            }
        } else if &c.detail_type == "private" {
            let target_id = c.user_id.ok_or(WQError::bad_param("user_id"))?;
            let target = target_id
                .parse()
                .map_err(|_| WQError::bad_param("user_id"))?;
            if let Some(chain) =
                MsgChainBuilder::private_chain_builder(&self.0, target, c.message.clone())
                    .build(&self.2)
                    .await
            {
                let receipt = self
                    .0
                    .send_friend_message(target, chain)
                    .await
                    .map_err(WQError::RQ)?;
                let message_id = receipt.seqs[0].to_string();
                let respc = SendMessageRespContent {
                    message_id,
                    time: receipt.time as f64,
                };
                let s_private = SPrivateMessage::receipt(
                    receipt,
                    target,
                    self.0.uin().await,
                    self.0.account_info.read().await.nickname.clone(),
                    c.message,
                );
                self.2.insert_private_message(&s_private);
                Ok(Resps::success(respc.into()))
            } else {
                Err(WQError::empty_message())
            }
        } else {
            Err(WQError::unsupported_action())
        }
    }

    async fn delete_message(&self, c: DeleteMessage, _ob: &OneBot) -> WQResult<Resps> {
        if let Some(m) = self.2.get_message(
            c.message_id
                .parse()
                .map_err(|_| WQError::bad_param("message_id"))?,
        ) {
            match m {
                SMessage::Private(p) => {
                    self.0
                        .recall_friend_message(p.from_uin, p.time as i64, p.seqs, p.rands)
                        .await
                        .map_err(WQError::RQ)?;
                }
                SMessage::Group(g) => {
                    self.0
                        .recall_group_message(g.group_code, g.seqs, g.rands)
                        .await
                        .map_err(WQError::RQ)?;
                }
            }
            Ok(Resps::empty_success())
        } else {
            Err(WQError::message_not_exist())
        }
    }

    async fn get_self_info(&self) -> WQResult<Resps> {
        Ok(Resps::success(
            UserInfoContent {
                user_id: self.0.uin().await.to_string(),
                nickname: self.0.account_info.read().await.nickname.clone(),
            }
            .into(),
        ))
    }
    async fn get_user_info(&self, c: GetUserInfo, _ob: &OneBot) -> WQResult<Resps> {
        let user_id: i64 = c
            .user_id
            .parse()
            .map_err(|_| WQError::bad_param("user_id"))?;
        let info = self
            .0
            .find_friend(user_id)
            .await
            .ok_or_else(|| WQError::friend_not_exist())?;
        Ok(Resps::success(
            UserInfoContent {
                user_id: info.uin.to_string(),
                nickname: info.nick.to_string(),
            }
            .into(),
        ))
    }
    async fn get_friend_list(&self) -> WQResult<Resps> {
        Ok(Resps::success(
            self.0
                .friends
                .read()
                .await
                .iter()
                .map(|i| UserInfoContent {
                    user_id: i.0.to_string(),
                    nickname: i.1.nick.to_string(),
                })
                .collect::<Vec<_>>()
                .into(),
        ))
    }
    async fn get_group_info(&self, c: GetGroupInfo, _ob: &OneBot) -> WQResult<Resps> {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| WQError::bad_param("group_id"))?;
        let info = self
            .0
            .find_group(group_id, true)
            .await
            .ok_or_else(|| WQError::group_not_exist())?;
        Ok(Resps::success(
            GroupInfoContent {
                group_id: info.info.uin.to_string(),
                group_name: info.info.name.to_string(),
            }
            .into(),
        ))
    }
    async fn get_group_list(&self) -> WQResult<Resps> {
        Ok(Resps::success(
            self.0
                .groups
                .read()
                .await
                .iter()
                .map(|i| GroupInfoContent {
                    group_id: i.0.to_string(),
                    group_name: i.1.info.name.clone(),
                })
                .collect::<Vec<_>>()
                .into(),
        ))
    }
    async fn get_group_member_list(&self, c: GetGroupMemberList) -> WQResult<Resps> {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| WQError::bad_param("group_id"))?;
        let group = self
            .0
            .find_group(group_id, true)
            .await
            .ok_or_else(|| WQError::group_not_exist())?;
        let v = group
            .members
            .read()
            .await
            .iter()
            .map(|i| UserInfoContent {
                user_id: i.uin.to_string(),
                nickname: i.nickname.clone(),
            })
            .collect::<Vec<_>>();
        Ok(Resps::success(v.into()))
    }
    async fn get_group_member_info(&self, c: GetGroupMemberInfo) -> WQResult<Resps> {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| WQError::bad_param("group_id"))?;
        let uin: i64 = c
            .user_id
            .parse()
            .map_err(|_| WQError::bad_param("user_id"))?;
        let group = self
            .0
            .find_group(group_id, true)
            .await
            .ok_or_else(|| WQError::group_not_exist())?;
        let list = group.members.read().await;
        let v: Vec<_> = list.iter().filter(|i| i.uin == uin).collect();
        if v.is_empty() {
            Err(WQError::group_member_not_exist())
        } else {
            Ok(Resps::success(
                UserInfoContent {
                    user_id: v[0].uin.to_string(),
                    nickname: v[0].nickname.clone(),
                }
                .into(),
            ))
        }
    }
    async fn set_group_name(&self, c: SetGroupName, _ob: &OneBot) -> WQResult<Resps> {
        self.0
            .update_group_name(
                c.group_id
                    .parse()
                    .map_err(|_| WQError::bad_param("group_id"))?,
                c.group_name,
            )
            .await
            .map_err(WQError::RQ)?;
        Ok(Resps::empty_success())
    }
    async fn leave_group(&self, c: LeaveGroup, _ob: &OneBot) -> WQResult<Resps> {
        self.0
            .group_quit(
                c.group_id
                    .parse()
                    .map_err(|_| WQError::bad_param("group_id"))?,
            )
            .await
            .map_err(WQError::RQ)?;
        Ok(Resps::empty_success())
    }
    async fn kick_group_member(&self, c: KickGroupMember, _ob: &OneBot) -> WQResult<Resps> {
        self.0
            .group_kick(
                c.group_id
                    .parse()
                    .map_err(|_| WQError::bad_param("group_id"))?,
                vec![c
                    .user_id
                    .parse()
                    .map_err(|_| WQError::bad_param("user_id"))?],
                "",
                false,
            )
            .await
            .map_err(WQError::RQ)?;
        Ok(Resps::empty_success())
    }
    async fn ban_group_member(
        &self,
        group_id: String,
        user_id: String,
        extra: ExtendedMap,
        _ob: &OneBot,
        unban: bool,
    ) -> WQResult<Resps> {
        use std::time::Duration;

        let duration: Duration = if unban {
            Duration::from_secs(0)
        } else {
            Duration::from_secs(extra.get("duration").map_or(Ok(60), |v| {
                match v.clone().downcast_int() {
                    Ok(v) => Ok(v as u64),
                    Err(_) => Err(WQError::bad_param("duration")),
                }
            })?)
        };
        self.0
            .group_mute(
                group_id
                    .parse()
                    .map_err(|_| WQError::bad_param("group_id"))?,
                user_id.parse().map_err(|_| WQError::bad_param("user_id"))?,
                duration,
            )
            .await
            .map_err(WQError::RQ)?;
        Ok(Resps::empty_success())
    }
    async fn set_group_admin(
        &self,
        group_id: String,
        user_id: String,
        _ob: &OneBot,
        unset: bool,
    ) -> WQResult<Resps> {
        self.0
            .group_set_admin(
                group_id
                    .parse()
                    .map_err(|_| WQError::bad_param("group_id"))?,
                user_id.parse().map_err(|_| WQError::bad_param("user_id"))?,
                !unset,
            )
            .await
            .map_err(WQError::RQ)?;
        Ok(Resps::empty_success())
    }
}
