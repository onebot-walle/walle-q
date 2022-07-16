use std::sync::Arc;

use async_trait::async_trait;
use cached::{Cached, SizedCache, TimedCache};
use once_cell::sync::OnceCell;
use ricq::structs::{FriendAudio, GroupAudio};
use ricq::Client;
use tokio::sync::Mutex;
use walle_core::action::*;
use walle_core::alt::ColoredAlt;
use walle_core::event::*;
use walle_core::prelude::*;
use walle_core::resp::*;
use walle_core::structs::{GroupInfo, SendMessageResp, UserInfo, Version};
use walle_core::util::SelfIds;
use walle_core::{ActionHandler, EventHandler, OneBot};

use crate::database::{Database, MessageId, WQDatabase};
use crate::error;
use crate::model::*;
use crate::parse::util::{
    decode_message_id, new_group_receipt_content, new_group_temp_receipt_content,
    new_private_receipt_content,
};
use crate::parse::{util::new_event, MsgChainBuilder, RQSendItem};

use self::file::FragmentFile;

mod file;
// pub(crate) mod v11;

pub struct Handler {
    pub(crate) client: OnceCell<Arc<ricq::Client>>,
    pub(crate) event_cache: Arc<Mutex<SizedCache<String, Event>>>,
    pub(crate) database: Arc<WQDatabase>,
    pub(crate) uploading_fragment: Mutex<TimedCache<String, FragmentFile>>,
}

#[async_trait]
impl SelfIds for Handler {
    async fn self_ids(&self) -> Vec<String> {
        vec![self.client.get().unwrap().uin().await.to_string()]
    }
}

impl GetStatus for Handler {
    fn get_status(&self) -> walle_core::structs::Status {
        walle_core::structs::Status {
            good: true, //todo
            online: self
                .client
                .get()
                .unwrap()
                .online
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

#[async_trait]
impl ActionHandler<Event, Action, Resp, 12> for Handler {
    type Config = (String, Option<String>, u8); // (uin, password, protcol)
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, 12>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp, 12> + Send + Sync + 'static,
    {
        let (qevent_tx, mut qevent_rx) = tokio::sync::mpsc::unbounded_channel();
        let qclient = Arc::new(Client::new_with_config(
            crate::config::load_device(&config.0, config.2).unwrap(),
            qevent_tx,
        ));
        let stream = tokio::net::TcpStream::connect(qclient.get_address())
            .await
            .unwrap();
        let _qcli = qclient.clone();
        let net = tokio::spawn(async move { _qcli.start(stream).await });
        self.client.set(qclient.clone()).ok();
        let event_cache = self.event_cache.clone();
        let database = self.database.clone();
        let ob = ob.clone();
        tokio::task::yield_now().await;
        crate::login::login(&qclient, &config.0, config.1.clone())
            .await
            .map_err(|e| WalleError::Other(e.to_string()))?;
        let mut tasks = vec![];
        let mut rx = ob.get_signal_rx()?;
        tasks.push(tokio::spawn(async move {
            while let Some(qevent) = qevent_rx.recv().await {
                if let Some(event) = crate::parse::qevent2event(qevent, &database).await {
                    tracing::info!(target: crate::WALLE_Q, "{}", event.colored_alt());
                    event_cache
                        .lock()
                        .await
                        .cache_set(event.id.clone(), event.clone());
                    ob.event_handler.call(event).await.ok();
                }
            }
        }));
        let _qcli = qclient.clone();
        tasks.push(tokio::spawn(async move {
            net.await.ok();
            crate::login::start_reconnect(&_qcli, &config.0, config.1).await;
        }));
        tasks.push(tokio::spawn(async move {
            rx.recv().await.ok();
            qclient.stop(ricq::client::NetworkStatus::NetworkOffline);
        }));
        Ok(tasks)
    }
    async fn call(&self, action: Action) -> WalleResult<Resp> {
        match self._handle(action).await {
            Ok(resp) => Ok(resp),
            Err(e) => Ok(e.into()),
        }
    }
}

use crate::model::WQAction;

impl Handler {
    async fn _handle(&self, action: Action) -> Result<Resp, RespError> {
        match WQAction::try_from(action).map_err(|e: WalleError| match e {
            WalleError::DeclareNotMatch(_, get) => resp_error::unsupported_action(get),
            WalleError::MapMissedKey(expect) => {
                resp_error::bad_param(format!("missing key {}", expect))
            }
            _ => unreachable!(),
        })? {
            WQAction::GetLatestEvents(c) => self.get_latest_events(c).await.map(Into::into),
            WQAction::GetSupportedActions {} => Self::get_supported_actions().map(Into::into),
            WQAction::GetStatus {} => Ok(self.get_status().into()),
            WQAction::GetVersion {} => Ok(Self::get_version().into()),

            WQAction::SendMessage(c) => self.send_message(c).await.map(Into::into),
            WQAction::DeleteMessage(c) => self.delete_message(c).await.map(Into::into),
            WQAction::GetMessage(c) => self.get_message(c).await.map(Into::into),

            WQAction::GetSelfInfo {} => self.get_self_info().await.map(Into::into),
            WQAction::GetUserInfo(c) => self.get_user_info(c).await.map(Into::into),
            WQAction::GetFriendList {} => self.get_friend_list().await.map(Into::into),

            WQAction::GetGroupInfo(c) => self.get_group_info(c).await.map(Into::into),
            WQAction::GetGroupList {} => self.get_group_list().await.map(Into::into),
            WQAction::GetGroupMemberInfo(c) => self.get_group_member_info(c).await.map(Into::into),
            WQAction::GetGroupMemberList(c) => self.get_group_member_list(c).await.map(Into::into),
            WQAction::SetGroupName(c) => self.set_group_name(c).await.map(Into::into),
            WQAction::LeaveGroup(c) => self.leave_group(c).await.map(Into::into),
            WQAction::KickGroupMember(c) => self.kick_group_member(c).await.map(Into::into),
            WQAction::BanGroupMember(c) => self
                .ban_group_member(c.group_id, c.user_id, c.duration)
                .await
                .map(Into::into),
            WQAction::UnbanGroupMember(c) => self
                .ban_group_member(c.group_id, c.user_id, 0)
                .await
                .map(Into::into),
            WQAction::SetGroupAdmin(c) => self
                .set_group_admin(c.group_id, c.user_id, false)
                .await
                .map(Into::into),
            WQAction::UnsetGroupAdmin(c) => self
                .set_group_admin(c.group_id, c.user_id, true)
                .await
                .map(Into::into),

            WQAction::UploadFile(c) => self.upload_file(c).await.map(Into::into),
            WQAction::UploadFileFragmented(c) => {
                self.upload_file_fragmented(c).await.map(Into::into)
            }
            WQAction::GetFile(c) => self.get_file(c).await.map(Into::into),
            WQAction::GetFileFragmented(c) => self.get_file_fragmented(c).await,
            WQAction::SetNewFriend(c) => self.set_new_friend(c).await.map(Into::into),
            WQAction::DeleteFriend(c) => self.delete_friend(c).await.map(Into::into),
            WQAction::GetNewFriendRequests {} => {
                self.get_new_friend_requests().await.map(Into::into)
            }
            WQAction::SetJoinGroup(c) => self.set_join_group_request(c).await.map(Into::into),
            WQAction::GetJoinGroupRequests {} => {
                self.get_join_group_requests().await.map(Into::into)
            }
            WQAction::SetGroupInvited(c) => self.set_group_invite(c).await.map(Into::into),
            WQAction::GetGroupInviteds {} => self.get_group_invites().await.map(Into::into),
        }
    }
}

type RespResult<T> = Result<T, RespError>;

impl Handler {
    pub fn get_client(&self) -> Result<&Arc<Client>, RespError> {
        self.client.get().ok_or(error::client_not_initialized(""))
    }
    async fn get_latest_events(&self, c: GetLatestEvents) -> Result<Vec<Event>, RespError> {
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
        Ok(events)
    }
    fn get_supported_actions() -> RespResult<Vec<&'static str>> {
        Ok(vec![
            "get_latest_events",
            "get_supported_actions",
            "get_status",
            "get_version",
            "send_message",
            "delete_message",
            "get_self_info",
            "get_user_info",
            "get_friend_list",
            "get_group_info",
            "get_group_list",
            "get_group_member_list",
            "get_group_member_info",
            "set_group_name",
            "kick_group_member",
            "ban_group_member",
            "unban_group_member",
            "set_group_admin",
            "unset_group_admin",
            "upload_file",
            "upload_file_fragmented",
            "get_file",
            "get_file_fragmented",
            // ext
            "set_new_friend",
            "delete_friend",
            "get_new_friend_request",
        ])
    }
    fn get_version() -> Version {
        Version {
            implt: crate::WALLE_Q.to_string(),
            platform: "qq".to_string(),
            version: crate::VERSION.to_string(),
            onebot_version: 12.to_string(),
        }
    }

    async fn send_message(&self, c: SendMessage) -> RespResult<SendMessageResp> {
        match c.detail_type.as_str() {
            "group" => {
                let group_id = c.group_id.ok_or_else(|| error::bad_param("group_id"))?;
                let group_code = group_id.parse().map_err(|_| error::bad_param("group_id"))?;
                let receipt = match MsgChainBuilder::group_chain_builder(
                    self.get_client()?,
                    group_code,
                    c.message.clone(),
                )
                .build(&self.database)
                .await?
                {
                    RQSendItem::Chain(chain) => self
                        .get_client()?
                        .send_group_message(group_code, chain)
                        .await
                        .map_err(error::rq_error)?,
                    RQSendItem::Forward(msgs) => self
                        .get_client()?
                        .send_group_forward_message(group_code, msgs)
                        .await
                        .map_err(error::rq_error)?,
                    RQSendItem::Voice(ptt) => self
                        .get_client()?
                        .send_group_audio(group_code, GroupAudio(ptt))
                        .await
                        .map_err(error::rq_error)?,
                };
                let time = receipt.time as f64;
                let cli = self.get_client()?;
                let event = new_event(
                    cli,
                    Some(time as f64),
                    new_group_receipt_content(cli, receipt, group_code, c.message).await,
                )
                .await;
                let respc = SendMessageResp {
                    message_id: event.message_id(),
                    time,
                };
                self.database.insert_message(&event);
                Ok(respc)
            }
            "group_temp" => {
                let group_id = c.group_id.ok_or_else(|| error::bad_param("group_id"))?;
                let group_code = group_id.parse().map_err(|_| error::bad_param("group_id"))?;
                let target_id = c.user_id.ok_or_else(|| error::bad_param("user_id"))?;
                let target = target_id.parse().map_err(|_| error::bad_param("user_id"))?;
                let receipt = match MsgChainBuilder::private_chain_builder(
                    self.get_client()?,
                    target,
                    c.message.clone(),
                )
                .build(&self.database)
                .await?
                {
                    RQSendItem::Chain(chain) => self
                        .get_client()?
                        .send_group_temp_message(group_code, target, chain)
                        .await
                        .map_err(error::rq_error)?,
                    RQSendItem::Forward(_) => return Err(resp_error::unsupported_param("forward")),
                    RQSendItem::Voice(_) => return Err(resp_error::unsupported_param("voice")),
                };
                let cli = self.get_client()?;
                let time = receipt.time as f64;
                let event = new_event(
                    cli,
                    Some(time),
                    new_group_temp_receipt_content(receipt, c.message, cli, group_code, target)
                        .await,
                )
                .await;
                let respc = SendMessageResp {
                    message_id: event.message_id(),
                    time,
                };
                self.database.insert_message(&event);
                Ok(respc)
            }
            "private" => {
                let target_id = c.user_id.ok_or_else(|| error::bad_param("user_id"))?;
                let target = target_id.parse().map_err(|_| error::bad_param("user_id"))?;
                let receipt = match MsgChainBuilder::private_chain_builder(
                    self.get_client()?,
                    target,
                    c.message.clone(),
                )
                .build(&self.database)
                .await?
                {
                    RQSendItem::Chain(chain) => self
                        .get_client()?
                        .send_friend_message(target, chain)
                        .await
                        .map_err(error::rq_error)?,
                    RQSendItem::Voice(ptt) => self
                        .get_client()?
                        .send_friend_audio(target, FriendAudio(ptt))
                        .await
                        .map_err(error::rq_error)?,
                    _ => return Err(resp_error::unsupported_segment("forward")),
                };
                let cli = self.get_client()?;
                let time = receipt.time as f64;
                let event = new_event(
                    cli,
                    Some(time),
                    new_private_receipt_content(cli, receipt, target, c.message).await,
                )
                .await;
                let respc = SendMessageResp {
                    message_id: event.message_id(),
                    time,
                };
                self.database.insert_message(&event);
                Ok(respc)
            }
            ty => Err(resp_error::unsupported_param(ty)),
        }
    }

    async fn delete_message(&self, c: DeleteMessage) -> RespResult<()> {
        let message = decode_message_id(&c.message_id)?;
        match message.3 {
            Some(time) => self
                .get_client()?
                .recall_friend_message(message.0, time as i64, message.1, message.2)
                .await
                .map_err(error::rq_error)?,
            None => self
                .get_client()?
                .recall_group_message(message.0, message.1, message.2)
                .await
                .map_err(error::rq_error)?,
        }
        Ok(())
    }

    async fn get_message(&self, c: GetMessage) -> RespResult<Event> {
        if let Some(m) = self.database.get_message::<Event>(
            &c.message_id
                .parse::<String>()
                .map_err(|_| error::bad_param("message_id"))?,
        ) {
            Ok(m)
        } else {
            Err(error::message_not_exist(c.message_id))
        }
    }

    async fn get_self_info(&self) -> RespResult<UserInfo> {
        Ok(UserInfo {
            user_id: self.get_client()?.uin().await.to_string(),
            nickname: self
                .get_client()?
                .account_info
                .read()
                .await
                .nickname
                .clone(),
        })
    }
    async fn get_user_info(&self, c: GetUserInfo) -> RespResult<UserInfo> {
        let user_id: i64 = c.user_id.parse().map_err(|_| error::bad_param("user_id"))?;
        let info = self
            .get_client()?
            .get_summary_info(user_id)
            .await
            .map_err(error::rq_error)?;
        Ok(UserInfo {
            user_id: info.uin.to_string(),
            nickname: info.nickname,
        })
    }
    async fn get_friend_list(&self) -> RespResult<Vec<UserInfo>> {
        Ok(self
            .get_client()?
            .get_friend_list()
            .await
            .map_err(error::rq_error)?
            .friends
            .iter()
            .map(|i| UserInfo {
                user_id: i.uin.to_string(),
                nickname: i.nick.to_string(),
            })
            .collect::<Vec<_>>())
    }
    async fn get_group_info(&self, c: GetGroupInfo) -> RespResult<GroupInfo> {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| error::bad_param("group_id"))?;
        let info = self
            .get_client()?
            .get_group_info(group_id)
            .await
            .map_err(error::rq_error)?
            .ok_or_else(|| error::group_not_exist(c.group_id))?;
        Ok(GroupInfo {
            group_id: info.uin.to_string(),
            group_name: info.name,
        })
    }
    async fn get_group_list(&self) -> RespResult<Vec<GroupInfo>> {
        Ok(self
            .get_client()?
            .get_group_list()
            .await
            .map_err(error::rq_error)?
            .into_iter()
            .map(|i| GroupInfo {
                group_id: i.code.to_string(),
                group_name: i.name,
            })
            .collect::<Vec<_>>())
    }
    async fn get_group_member_list(&self, c: GetGroupMemberList) -> RespResult<Vec<UserInfo>> {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| error::bad_param("group_id"))?;
        let group = self
            .get_client()?
            .get_group_info(group_id)
            .await
            .map_err(error::rq_error)?
            .ok_or_else(|| error::group_not_exist(c.group_id))?;

        let v = self
            .get_client()?
            .get_group_member_list(group_id, group.owner_uin)
            .await
            .map_err(error::rq_error)?
            .iter()
            .map(|i| UserInfo {
                user_id: i.uin.to_string(),
                nickname: i.nickname.clone(),
            })
            .collect::<Vec<_>>();
        Ok(v)
    }
    async fn get_group_member_info(&self, c: GetGroupMemberInfo) -> RespResult<UserInfo> {
        let group_id: i64 = c
            .group_id
            .parse()
            .map_err(|_| error::bad_param("group_id"))?;
        let uin: i64 = c.user_id.parse().map_err(|_| error::bad_param("user_id"))?;
        let member = self
            .get_client()?
            .get_group_member_info(group_id, uin)
            .await
            .map_err(error::rq_error)?;
        Ok(UserInfo {
            user_id: member.uin.to_string(),
            nickname: member.nickname,
        })
    }
    async fn set_group_name(&self, c: SetGroupName) -> RespResult<()> {
        self.get_client()?
            .update_group_name(
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
                c.group_name,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(())
    }
    async fn leave_group(&self, c: LeaveGroup) -> RespResult<()> {
        self.get_client()?
            .group_quit(
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(())
    }
    async fn kick_group_member(&self, c: KickGroupMember) -> RespResult<()> {
        self.get_client()?
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
        Ok(())
    }
    async fn ban_group_member(
        &self,
        group_id: String,
        user_id: String,
        duration: u32,
    ) -> RespResult<()> {
        use std::time::Duration;

        let duration = Duration::from_secs(duration as u64);
        self.get_client()?
            .group_mute(
                group_id.parse().map_err(|_| error::bad_param("group_id"))?,
                user_id.parse().map_err(|_| error::bad_param("user_id"))?,
                duration,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(())
    }
    async fn set_group_admin(
        &self,
        group_id: String,
        user_id: String,
        unset: bool,
    ) -> RespResult<()> {
        self.get_client()?
            .group_set_admin(
                group_id.parse().map_err(|_| error::bad_param("group_id"))?,
                user_id.parse().map_err(|_| error::bad_param("user_id"))?,
                !unset,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(())
    }
}

impl Handler {
    async fn set_new_friend(&self, c: SetNewFriend) -> RespResult<()> {
        self.get_client()?
            .solve_friend_system_message(
                c.request_id,
                c.user_id.parse().map_err(|_| error::bad_param("user_id"))?,
                c.accept,
            )
            .await
            .map_err(error::rq_error)?;
        Ok(())
    }
    async fn delete_friend(&self, c: DeleteFriend) -> RespResult<()> {
        self.get_client()?
            .delete_friend(c.user_id.parse().map_err(|_| error::bad_param("user_id"))?)
            .await
            .map_err(error::rq_error)?;
        Ok(())
    }
    async fn get_new_friend_requests(&self) -> RespResult<Vec<NewFriend>> {
        Ok(self
            .get_client()?
            .get_friend_system_messages()
            .await
            .map_err(error::rq_error)?
            .requests
            .into_iter()
            .map(|r| NewFriend {
                request_id: r.msg_seq,
                user_id: r.req_uin.to_string(),
                user_name: r.req_nick,
                message: r.message,
            })
            .collect::<Vec<_>>())
    }
    async fn set_join_group_request(&self, c: SetJoinGroup) -> RespResult<()> {
        self.get_client()?
            .solve_group_system_message(
                c.request_id,
                c.user_id.parse().map_err(|_| error::bad_param("user_id"))?,
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
                false,
                false,
                c.accept,
                c.block.unwrap_or_default(),
                c.message.unwrap_or_default(),
            )
            .await
            .map_err(|e| error::rq_error(e))?;
        Ok(())
    }
    async fn get_join_group_requests(&self) -> RespResult<Vec<JoinGroup>> {
        Ok(self
            .get_client()?
            .get_all_group_system_messages()
            .await
            .map_err(error::rq_error)?
            .join_group_requests
            .into_iter()
            .map(|req| JoinGroup {
                request_id: req.msg_seq,
                user_id: req.req_uin.to_string(),
                user_name: req.req_nick,
                group_id: req.group_code.to_string(),
                group_name: req.group_name,
                message: req.message,
                suspicious: req.suspicious,
                invitor_id: req.invitor_uin.map(|uin| uin.to_string()),
                invitor_name: req.invitor_nick,
            })
            .collect())
    }
    async fn set_group_invite(&self, c: SetGroupInvited) -> RespResult<()> {
        self.get_client()?
            .solve_group_system_message(
                c.request_id,
                self.get_client()?.uin().await,
                c.group_id
                    .parse()
                    .map_err(|_| error::bad_param("group_id"))?,
                false,
                true,
                c.accept,
                false,
                String::default(),
            )
            .await
            .map_err(|e| error::rq_error(e))?;
        Ok(())
    }
    async fn get_group_invites(&self) -> RespResult<Vec<GroupInvite>> {
        Ok(self
            .get_client()?
            .get_all_group_system_messages()
            .await
            .map_err(error::rq_error)?
            .self_invited
            .into_iter()
            .map(|i| GroupInvite {
                request_id: i.msg_seq,
                group_id: i.group_code.to_string(),
                group_name: i.group_name,
                invitor_id: i.invitor_uin.to_string(),
                invitor_name: i.invitor_nick,
            })
            .collect())
    }
}
