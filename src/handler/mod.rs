use std::sync::Arc;

use async_trait::async_trait;
use cached::SizedCache;
use tokio::sync::Mutex;
use walle_core::action::{
    DeleteMessage, GetGroupInfo, GetGroupMemberInfo, GetGroupMemberList, GetLatestEvents,
    GetMessage, GetUserInfo, KickGroupMember, LeaveGroup, SendMessage, SetGroupName,
};
use walle_core::resp::{error_builder, RespError};
use walle_core::{
    impls::StandardOneBot,
    resp::{
        GroupInfoContent, SendMessageRespContent, StatusContent, UserInfoContent, VersionContent,
    },
    ActionHandler, ColoredAlt, RespContent, Resps, StandardAction, StandardEvent,
};
use walle_core::{ExtendedMap, MessageContent};

use crate::database::{Database, SGroupMessage, SMessage, SPrivateMessage, WQDatabase};
use crate::error;
use crate::parse::MsgChainBuilder;
use crate::WQResp;

type WQRespResult = Result<WQResp, RespError>;

mod file;
pub(crate) mod v11;

pub(crate) struct Handler(
    pub(crate) Arc<ricq::Client>,
    pub(crate) Arc<Mutex<SizedCache<String, StandardEvent>>>,
    pub(crate) Arc<WQDatabase>,
);

pub(crate) type OneBot = StandardOneBot<Handler>;

#[async_trait]
impl ActionHandler<StandardAction, WQResp, OneBot> for Handler {
    type Error = RespError;
    async fn handle(&self, action: StandardAction, ob: &OneBot) -> WQRespResult {
        if let Some(alt) = action.colored_alt() {
            tracing::info!(target: crate::WALLE_Q, "{}", alt);
        }
        match action {
            StandardAction::GetLatestEvents(c) => self.get_latest_events(c, ob).await,
            StandardAction::GetSupportedActions(_) => Self::get_supported_actions(),
            StandardAction::GetStatus(_) => self.get_status(),
            StandardAction::GetVersion(_) => Self::get_version(),

            StandardAction::SendMessage(c) => self.send_message(c, ob).await,
            StandardAction::DeleteMessage(c) => self.delete_message(c, ob).await,
            StandardAction::GetMessage(c) => self.get_message(c, ob).await,

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
            // StandardAction::UploadFileFragmented(_c) => Err(WQError::unsupported_action()),
            StandardAction::GetFile(c) => self.get_file(c, ob).await,
            // StandardAction::GetFileFragmented(_c) => Err(WQError::unsupported_action()),
            _ => Err(error_builder::unsupported_action()),
        }
    }
}

impl Handler {
    async fn get_latest_events(&self, c: GetLatestEvents, _ob: &OneBot) -> WQRespResult {
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
        Ok(Resps::success(RespContent::LatestEvents(events)))
    }
    fn get_supported_actions() -> WQRespResult {
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
    fn get_version() -> WQRespResult {
        Ok(Resps::success(
            VersionContent {
                r#impl: crate::WALLE_Q.to_string(),
                platform: "qq".to_string(),
                version: crate::VERSION.to_string(),
                onebot_version: OneBot::onebot_version().to_string(),
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }
    fn get_status(&self) -> WQRespResult {
        Ok(Resps::success(
            StatusContent {
                good: true,
                online: self.0.online.load(std::sync::atomic::Ordering::Relaxed),
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }

    async fn send_message(&self, c: SendMessage, ob: &OneBot) -> WQRespResult {
        if &c.detail_type == "group" {
            let group_id = c.group_id.ok_or_else(|| error::bad_param("group_id"))?;
            let group_code = group_id.parse().map_err(|_| error::bad_param("group_id"))?;
            if let Some(chain) =
                MsgChainBuilder::group_chain_builder(&self.0, group_code, c.message.clone())
                    .build(&self.2)
                    .await
            {
                let receipt = self
                    .0
                    .send_group_message(group_code, chain)
                    .await
                    .map_err(error::rq_error)?;
                let message_id = receipt.seqs[0].to_string();
                let respc = SendMessageRespContent {
                    message_id: message_id.clone(),
                    time: receipt.time as f64,
                    extra: ExtendedMap::default(),
                };
                let s_group = SGroupMessage::receipt(
                    receipt.clone(),
                    group_code,
                    ob.new_event(
                        MessageContent::new_group_message_content(
                            c.message,
                            message_id,
                            ob.self_id.read().await.clone(),
                            group_id,
                            [].into(),
                        )
                        .into(),
                        receipt.time as f64,
                    )
                    .await,
                );
                self.2.insert_group_message(&s_group);
                Ok(Resps::success(respc.into()))
            } else {
                Err(error::empty_message())
            }
        } else if &c.detail_type == "private" {
            let target_id = c.user_id.ok_or_else(|| error::bad_param("user_id"))?;
            let target = target_id.parse().map_err(|_| error::bad_param("user_id"))?;
            if let Some(chain) =
                MsgChainBuilder::private_chain_builder(&self.0, target, c.message.clone())
                    .build(&self.2)
                    .await
            {
                let receipt = self
                    .0
                    .send_friend_message(target, chain)
                    .await
                    .map_err(error::rq_error)?;
                let message_id = receipt.seqs[0].to_string();
                let respc = SendMessageRespContent {
                    message_id: message_id.clone(),
                    time: receipt.time as f64,
                    extra: ExtendedMap::default(),
                };
                let s_private = SPrivateMessage::receipt(
                    receipt.clone(),
                    target,
                    ob.new_event(
                        MessageContent::new_private_message_content(
                            c.message,
                            message_id,
                            ob.self_id.read().await.clone(),
                            [].into(),
                        )
                        .into(),
                        receipt.time as f64,
                    )
                    .await,
                );
                self.2.insert_private_message(&s_private);
                Ok(Resps::success(respc.into()))
            } else {
                Err(error::empty_message())
            }
        } else {
            Err(error_builder::unsupported_action())
        }
    }

    async fn delete_message(&self, c: DeleteMessage, _ob: &OneBot) -> WQRespResult {
        if let Some(m) = self.2.get_message(
            c.message_id
                .parse()
                .map_err(|_| error::bad_param("message_id"))?,
        ) {
            match m {
                SMessage::Private(p) => {
                    self.0
                        .recall_friend_message(p.target_id, p.time as i64, p.seqs, p.rands)
                        .await
                        .map_err(error::rq_error)?;
                }
                SMessage::Group(g) => {
                    self.0
                        .recall_group_message(g.group_code, g.seqs, g.rands)
                        .await
                        .map_err(error::rq_error)?;
                }
            }
            Ok(Resps::empty_success())
        } else {
            Err(error::message_not_exist())
        }
    }

    async fn get_message(&self, c: GetMessage, _ob: &OneBot) -> WQRespResult {
        if let Some(m) = self.2.get_message(
            c.message_id
                .parse()
                .map_err(|_| error::bad_param("message_id"))?,
        ) {
            Ok(Resps::success(RespContent::MessageEvent(m.event())))
        } else {
            Err(error::message_not_exist())
        }
    }

    async fn get_self_info(&self) -> WQRespResult {
        Ok(Resps::success(
            UserInfoContent {
                user_id: self.0.uin().await.to_string(),
                nickname: self.0.account_info.read().await.nickname.clone(),
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }
    async fn get_user_info(&self, c: GetUserInfo, _ob: &OneBot) -> WQRespResult {
        let user_id: i64 = c.user_id.parse().map_err(|_| error::bad_param("user_id"))?;
        let info = self
            .0
            .get_summary_info(user_id)
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::success(
            UserInfoContent {
                user_id: info.uin.to_string(),
                nickname: info.nickname,
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }
    async fn get_friend_list(&self) -> WQRespResult {
        Ok(Resps::success(
            self.0
                .get_friend_list()
                .await
                .map_err(error::rq_error)?
                .friends
                .iter()
                .map(|i| UserInfoContent {
                    user_id: i.uin.to_string(),
                    nickname: i.nick.to_string(),
                    extra: ExtendedMap::default(),
                })
                .collect::<Vec<_>>()
                .into(),
        ))
    }
    async fn get_group_info(&self, c: GetGroupInfo, _ob: &OneBot) -> WQRespResult {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| error::bad_param("group_id"))?;
        let info = self
            .0
            .get_group_info(group_id)
            .await
            .map_err(error::rq_error)?
            .ok_or_else(error::group_not_exist)?;
        Ok(Resps::success(
            GroupInfoContent {
                group_id: info.uin.to_string(),
                group_name: info.name,
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }
    async fn get_group_list(&self) -> WQRespResult {
        Ok(Resps::success(
            self.0
                .get_group_list()
                .await
                .map_err(error::rq_error)?
                .into_iter()
                .map(|i| GroupInfoContent {
                    group_id: i.code.to_string(),
                    group_name: i.name,
                    extra: ExtendedMap::default(),
                })
                .collect::<Vec<_>>()
                .into(),
        ))
    }
    async fn get_group_member_list(&self, c: GetGroupMemberList) -> WQRespResult {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| error::bad_param("group_id"))?;
        let group = self
            .0
            .get_group_info(group_id)
            .await
            .map_err(error::rq_error)?
            .ok_or_else(error::group_not_exist)?;

        let v = self
            .0
            .get_group_member_list(group_id, group.owner_uin)
            .await
            .map_err(error::rq_error)?
            .iter()
            .map(|i| UserInfoContent {
                user_id: i.uin.to_string(),
                nickname: i.nickname.clone(),
                extra: ExtendedMap::default(),
            })
            .collect::<Vec<_>>();
        Ok(Resps::success(v.into()))
    }
    async fn get_group_member_info(&self, c: GetGroupMemberInfo) -> WQRespResult {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| error::bad_param("group_id"))?;
        let uin: i64 = c.user_id.parse().map_err(|_| error::bad_param("user_id"))?;
        let member = self
            .0
            .get_group_member_info(group_id, uin)
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::success(
            UserInfoContent {
                user_id: member.uin.to_string(),
                nickname: member.nickname,
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }
    async fn set_group_name(&self, c: SetGroupName, _ob: &OneBot) -> WQRespResult {
        self.0
            .update_group_name(
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
                c.group_name,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn leave_group(&self, c: LeaveGroup, _ob: &OneBot) -> WQRespResult {
        self.0
            .group_quit(
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn kick_group_member(&self, c: KickGroupMember, _ob: &OneBot) -> WQRespResult {
        self.0
            .group_kick(
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
                vec![c.user_id.parse().map_err(|_| error::bad_param("user_id"))?],
                "",
                false,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn ban_group_member(
        &self,
        group_id: String,
        user_id: String,
        extra: ExtendedMap,
        _ob: &OneBot,
        unban: bool,
    ) -> WQRespResult {
        use std::time::Duration;

        let duration: Duration = if unban {
            Duration::from_secs(0)
        } else {
            Duration::from_secs(extra.get("duration").map_or(Ok(60), |v| {
                match v.clone().downcast_int() {
                    Ok(v) => Ok(v as u64),
                    Err(_) => Err(error::bad_param("duration")),
                }
            })?)
        };
        self.0
            .group_mute(
                group_id.parse().map_err(|_| error::bad_param("group_id"))?,
                user_id.parse().map_err(|_| error::bad_param("user_id"))?,
                duration,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn set_group_admin(
        &self,
        group_id: String,
        user_id: String,
        _ob: &OneBot,
        unset: bool,
    ) -> WQRespResult {
        self.0
            .group_set_admin(
                group_id.parse().map_err(|_| error::bad_param("group_id"))?,
                user_id.parse().map_err(|_| error::bad_param("user_id"))?,
                !unset,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
}
