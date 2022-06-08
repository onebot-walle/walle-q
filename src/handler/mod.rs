use std::sync::Arc;

use async_trait::async_trait;
use cached::{SizedCache, TimedCache};
use tokio::sync::Mutex;
use walle_core::{action::*, ExtendedValue};
use walle_core::{extended_value, resp::*};
use walle_core::{
    ActionHandler, ColoredAlt, ExtendedMap, MessageContent, RespContent, Resps, StandardAction,
    StandardEvent,
};

use crate::database::{Database, SGroupMessage, SMessage, SPrivateMessage, WQDatabase};
use crate::error;
use crate::extra::*;
use crate::parse::{MsgChainBuilder, RQSendable};
use crate::{OneBot, WQResp};

use self::file::FragmentFile;

type WQRespResult = Result<WQResp, RespError>;

mod file;
pub(crate) mod v11;

pub(crate) struct Handler {
    pub(crate) client: Arc<ricq::Client>,
    pub(crate) event_cache: Arc<Mutex<SizedCache<String, StandardEvent>>>,
    pub(crate) database: Arc<WQDatabase>,
    pub(crate) uploading_fragment: Mutex<TimedCache<String, FragmentFile>>,
}

impl ActionHandler<WQAction, WQResp, OneBot> for Handler {
    type Error = RespError;
    fn handle<'life0, 'life1, 'async_trait>(
        &'life0 self,
        action: WQAction,
        ob: &'life1 OneBot,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = WQRespResult> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        match action {
            WQAction::Standard(standard) => self.handle(standard, ob),
            WQAction::Extra(extended) => self.handle(extended, ob),
        }
    }
}

#[async_trait]
impl ActionHandler<WQExtraAction, WQResp, OneBot> for Handler {
    type Error = RespError;
    async fn handle(&self, action: WQExtraAction, _ob: &OneBot) -> WQRespResult {
        match action {
            WQExtraAction::SetNewFriend(c) => self.set_new_friend(c).await,
            WQExtraAction::DeleteFriend(c) => self.delete_friend(c).await,
            WQExtraAction::GetNewFriendRequest(_) => self.get_new_friend_request().await,
        }
    }
}

#[async_trait]
impl ActionHandler<StandardAction, WQResp, OneBot> for Handler {
    type Error = RespError;
    async fn handle(&self, action: StandardAction, ob: &OneBot) -> WQRespResult {
        if let Some(alt) = action.colored_alt() {
            tracing::info!(target: crate::WALLE_Q, "{}", alt);
        }
        match action {
            StandardAction::GetLatestEvents(c) => self.get_latest_events(c).await,
            StandardAction::GetSupportedActions(_) => Self::get_supported_actions(),
            StandardAction::GetStatus(_) => self.get_status(),
            StandardAction::GetVersion(_) => Self::get_version(),

            StandardAction::SendMessage(c) => self.send_message(c, ob).await,
            StandardAction::DeleteMessage(c) => self.delete_message(c).await,
            StandardAction::GetMessage(c) => self.get_message(c).await,

            StandardAction::GetSelfInfo(_) => self.get_self_info().await,
            StandardAction::GetUserInfo(c) => self.get_user_info(c).await,
            StandardAction::GetFriendList(_) => self.get_friend_list().await,

            StandardAction::GetGroupInfo(c) => self.get_group_info(c).await,
            StandardAction::GetGroupList(_) => self.get_group_list().await,
            StandardAction::GetGroupMemberInfo(c) => self.get_group_member_info(c).await,
            StandardAction::GetGroupMemberList(c) => self.get_group_member_list(c).await,
            StandardAction::SetGroupName(c) => self.set_group_name(c).await,
            StandardAction::LeaveGroup(c) => self.leave_group(c).await,
            StandardAction::KickGroupMember(c) => self.kick_group_member(c).await,
            StandardAction::BanGroupMember(c) => {
                self.ban_group_member(c.group_id, c.user_id, c.extra, false)
                    .await
            }
            StandardAction::UnbanGroupMember(c) => {
                self.ban_group_member(c.group_id, c.user_id, c.extra, true)
                    .await
            }
            StandardAction::SetGroupAdmin(c) => {
                self.set_group_admin(c.group_id, c.user_id, false).await
            }
            StandardAction::UnsetGroupAdmin(c) => {
                self.set_group_admin(c.group_id, c.user_id, true).await
            }

            StandardAction::UploadFile(c) => self.upload_file(c, ob).await,
            StandardAction::UploadFileFragmented(c) => self.upload_file_fragmented(c, ob).await,
            StandardAction::GetFile(c) => self.get_file(c, ob).await,
            StandardAction::GetFileFragmented(c) => self.get_file_fragmented(c, ob).await,
            _ => Err(error_builder::unsupported_action()),
        }
    }
}

impl Handler {
    async fn get_latest_events(&self, c: GetLatestEvents) -> WQRespResult {
        let get = || async {
            self.event_cache
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
            "upload_file_fragmented".into(),
            "get_file".into(),
            "get_file_fragmented".into(),
            // ext
            "set_new_friend".into(),
            "delete_friend".into(),
            "get_new_friend_request".into(),
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
                online: self
                    .client
                    .online
                    .load(std::sync::atomic::Ordering::Relaxed),
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }

    async fn send_message(&self, c: SendMessage, ob: &OneBot) -> WQRespResult {
        if &c.detail_type == "group" {
            let group_id = c.group_id.ok_or_else(|| error::bad_param("group_id"))?;
            let group_code = group_id.parse().map_err(|_| error::bad_param("group_id"))?;
            if let Some(s) =
                MsgChainBuilder::group_chain_builder(&self.client, group_code, c.message.clone())
                    .build(&self.database)
                    .await?
            {
                let receipt = match s {
                    RQSendable::Chain(chain) => self
                        .client
                        .send_group_message(group_code, chain)
                        .await
                        .map_err(error::rq_error)?,
                    RQSendable::Forward(msgs) => self
                        .client
                        .send_group_forward_message(group_code, msgs)
                        .await
                        .map_err(error::rq_error)?,
                };
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
                self.database.insert_group_message(&s_group);
                Ok(Resps::success(respc.into()))
            } else {
                Err(error::empty_message())
            }
        } else if &c.detail_type == "private" {
            let target_id = c.user_id.ok_or_else(|| error::bad_param("user_id"))?;
            let target = target_id.parse().map_err(|_| error::bad_param("user_id"))?;
            if let Some(s) =
                MsgChainBuilder::private_chain_builder(&self.client, target, c.message.clone())
                    .build(&self.database)
                    .await?
            {
                let receipt = match s {
                    RQSendable::Chain(chain) => self
                        .client
                        .send_friend_message(target, chain)
                        .await
                        .map_err(error::rq_error)?,
                    RQSendable::Forward(_) => return Err(error::unsupported_param("forward")),
                };
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
                self.database.insert_private_message(&s_private);
                Ok(Resps::success(respc.into()))
            } else {
                Err(error::empty_message())
            }
        } else {
            Err(error_builder::unsupported_action())
        }
    }

    async fn delete_message(&self, c: DeleteMessage) -> WQRespResult {
        if let Some(m) = self.database.get_message(
            c.message_id
                .parse()
                .map_err(|_| error::bad_param("message_id"))?,
        ) {
            match m {
                SMessage::Private(p) => {
                    self.client
                        .recall_friend_message(p.target_id, p.time as i64, p.seqs, p.rands)
                        .await
                        .map_err(error::rq_error)?;
                }
                SMessage::Group(g) => {
                    self.client
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

    async fn get_message(&self, c: GetMessage) -> WQRespResult {
        if let Some(m) = self.database.get_message(
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
                user_id: self.client.uin().await.to_string(),
                nickname: self.client.account_info.read().await.nickname.clone(),
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }
    async fn get_user_info(&self, c: GetUserInfo) -> WQRespResult {
        let user_id: i64 = c.user_id.parse().map_err(|_| error::bad_param("user_id"))?;
        let info = self
            .client
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
            self.client
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
    async fn get_group_info(&self, c: GetGroupInfo) -> WQRespResult {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| error::bad_param("group_id"))?;
        let info = self
            .client
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
            self.client
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
            .client
            .get_group_info(group_id)
            .await
            .map_err(error::rq_error)?
            .ok_or_else(error::group_not_exist)?;

        let v = self
            .client
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
            .client
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
    async fn set_group_name(&self, c: SetGroupName) -> WQRespResult {
        self.client
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
    async fn leave_group(&self, c: LeaveGroup) -> WQRespResult {
        self.client
            .group_quit(
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn kick_group_member(&self, c: KickGroupMember) -> WQRespResult {
        self.client
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
        self.client
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
        unset: bool,
    ) -> WQRespResult {
        self.client
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

impl Handler {
    async fn set_new_friend(&self, c: SetNewFriend) -> WQRespResult {
        self.client
            .solve_friend_system_message(
                c.request_id,
                c.user_id.parse().map_err(|_| error::bad_param("user_id"))?,
                c.accept,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn delete_friend(&self, c: DeleteFriend) -> WQRespResult {
        self.client
            .delete_friend(c.user_id.parse().map_err(|_| error::bad_param("user_id"))?)
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn get_new_friend_request(&self) -> WQRespResult {
        Ok(Resps::success(
            ExtendedValue::List(
                self.client
                    .get_friend_system_messages()
                    .await
                    .map_err(error::rq_error)?
                    .requests
                    .into_iter()
                    .map(|r| {
                        extended_value!({
                            "request_id": r.msg_seq,
                            "user_id": r.req_uin,
                            "user_name": r.req_nick,
                            "message": r.message,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .into(),
        ))
    }
}
