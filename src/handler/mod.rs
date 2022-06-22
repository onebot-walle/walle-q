use std::sync::Arc;

use async_trait::async_trait;
use cached::{Cached, SizedCache, TimedCache};
use once_cell::sync::OnceCell;
use ricq::structs::{FriendAudio, GroupAudio};
use ricq::Client;
use tokio::sync::Mutex;
use walle_core::onebot::{ActionHandler, EventHandler, OneBot};
use walle_core::{action::*, ExtendedValue, MessageAlt};
use walle_core::{extended_value, resp::*};
use walle_core::{ColoredAlt, ExtendedMap, MessageContent, RespContent, Resps, StandardAction};

use crate::config::QQConfig;
use crate::database::{Database, SGroupMessage, SMessage, SPrivateMessage, WQDatabase};
use crate::error;
use crate::extra::*;
use crate::parse::{new_event, MsgChainBuilder, RQSendItem};
use crate::WQResp;

use self::file::FragmentFile;

type WQRespResult = Result<WQResp, RespError>;

mod file;
// pub(crate) mod v11;

pub(crate) struct Handler {
    pub(crate) client: OnceCell<Arc<ricq::Client>>,
    pub(crate) event_cache: Arc<Mutex<SizedCache<String, WQEvent>>>,
    pub(crate) database: Arc<WQDatabase>,
    pub(crate) uploading_fragment: Mutex<TimedCache<String, FragmentFile>>,
}

#[async_trait]
impl ActionHandler<WQEvent, WQAction, WQResp, 12> for Handler {
    type Config = QQConfig;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, 12>>,
        config: Self::Config,
    ) -> walle_core::WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
        EH: EventHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
    {
        let (qevent_tx, mut qevent_rx) = tokio::sync::mpsc::unbounded_channel();
        let qclient = Arc::new(Client::new_with_config(
            crate::config::load_device(&config).unwrap(),
            qevent_tx,
        ));
        let stream = tokio::net::TcpStream::connect(qclient.get_address())
            .await
            .unwrap();
        let _qcli = qclient.clone();
        let _net = tokio::spawn(async move { _qcli.start(stream).await });
        let event_cache = self.event_cache.clone();
        let database = self.database.clone();
        let ob = ob.clone();
        tokio::task::yield_now().await;
        crate::login::login(&qclient, &config).await.unwrap();
        let mut tasks = vec![];
        tasks.push(tokio::spawn(async move {
            while let Some(qevent) = qevent_rx.recv().await {
                if let Some(event) = crate::parse::qevent2event(qevent, &database).await {
                    if let Some(alt) = event.colored_alt() {
                        tracing::info!(target: crate::WALLE_Q, "{}", alt);
                    }
                    event_cache
                        .lock()
                        .await
                        .cache_set(event.id.clone(), event.clone());
                    ob.event_handler.call(event, &ob).await
                }
            }
        }));
        Ok(tasks)
    }
    async fn call<AH, EH>(
        &self,
        action: WQAction,
        _ob: &OneBot<AH, EH, 12>,
    ) -> walle_core::WalleResult<WQResp>
    where
        AH: ActionHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
        EH: EventHandler<WQEvent, WQAction, WQResp, 12> + Send + Sync + 'static,
    {
        match self._handle(action).await {
            Ok(resp) => Ok(resp),
            Err(e) => Ok(e.into()),
        }
    }
}

impl Handler {
    async fn _handle(&self, action: WQAction) -> WQRespResult {
        match action {
            WQAction::Standard(standard) => match standard {
                StandardAction::GetLatestEvents(c) => self.get_latest_events(c).await,
                StandardAction::GetSupportedActions(_) => Self::get_supported_actions(),
                StandardAction::GetStatus(_) => self.get_status(),
                StandardAction::GetVersion(_) => Self::get_version(),

                StandardAction::SendMessage(c) => self.send_message(c).await,
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

                StandardAction::UploadFile(c) => self.upload_file(c).await,
                StandardAction::UploadFileFragmented(c) => self.upload_file_fragmented(c).await,
                StandardAction::GetFile(c) => self.get_file(c).await,
                StandardAction::GetFileFragmented(c) => self.get_file_fragmented(c).await,
                action => Err(error_builder::unsupported_action(action.action_type())),
            },
            WQAction::Extra(extra) => match extra {
                WQExtraAction::SetNewFriend(c) => self.set_new_friend(c).await,
                WQExtraAction::DeleteFriend(c) => self.delete_friend(c).await,
                WQExtraAction::GetNewFriendRequests(_) => self.get_new_friend_requests().await,
                WQExtraAction::SetJoinGroup(c) => self.set_join_group_request(c).await,
                WQExtraAction::GetJoinGroupRequests(_) => self.get_join_group_requests().await,
                WQExtraAction::SetGroupInvited(c) => self.set_group_invite(c).await,
                WQExtraAction::GetGroupInviteds(_) => self.get_group_invites().await,
            },
        }
    }
}

impl Handler {
    fn get_client(&self) -> Result<&Arc<Client>, RespError> {
        self.client.get().ok_or(error::client_not_initialized(""))
    }
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
                onebot_version: 12.to_string(),
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
                    .get_client()?
                    .online
                    .load(std::sync::atomic::Ordering::Relaxed),
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }

    async fn send_message(&self, c: SendMessage) -> WQRespResult {
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
                let message_id = receipt.seqs[0].to_string();
                let respc = SendMessageRespContent {
                    message_id: message_id.clone(),
                    time: receipt.time as f64,
                    extra: ExtendedMap::default(),
                };
                let s_group = SGroupMessage::receipt(
                    receipt.clone(),
                    group_code,
                    new_event(
                        self.get_client()?,
                        Some(receipt.time as f64),
                        MessageContent::<WQMEDetail> {
                            detail: WQMEDetail::Group {
                                sub_type: "".to_string(),
                                group_id,
                                group_name: "".to_string(),
                                user_name: self
                                    .get_client()?
                                    .account_info
                                    .read()
                                    .await
                                    .nickname
                                    .clone(),
                            },
                            message_id,
                            alt_message: c.message.alt(),
                            message: c.message,
                            user_id: self.get_client()?.uin().await.to_string(),
                        }
                        .into(),
                    )
                    .await,
                );
                self.database.insert_group_message(&s_group);
                Ok(Resps::success(respc.into()))
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
                    RQSendItem::Forward(_) => {
                        return Err(error_builder::unsupported_param("forward"))
                    }
                    RQSendItem::Voice(_) => return Err(error_builder::unsupported_param("voice")),
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
                    new_event(
                        self.get_client()?,
                        Some(receipt.time as f64),
                        MessageContent {
                            alt_message: c.message.alt(),
                            message: c.message,
                            message_id: receipt.seqs[0].to_string(),
                            user_id: self.get_client()?.uin().await.to_string(),
                            detail: WQMEDetail::GroupTemp {
                                sub_type: "".to_string(),
                                user_name: self
                                    .get_client()?
                                    .account_info
                                    .read()
                                    .await
                                    .nickname
                                    .clone(),
                                group_id,
                            },
                        }
                        .into(),
                    )
                    .await,
                );
                self.database.insert_private_message(&s_private);
                Ok(Resps::success(respc.into()))
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
                    _ => return Err(error_builder::unsupported_segment("forward")),
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
                    new_event(
                        self.get_client()?,
                        Some(receipt.time as f64),
                        MessageContent::<WQMEDetail> {
                            detail: WQMEDetail::Private {
                                sub_type: "".to_string(),
                                user_name: self
                                    .get_client()?
                                    .account_info
                                    .read()
                                    .await
                                    .nickname
                                    .clone(),
                            },
                            message_id,
                            alt_message: c.message.alt(),
                            message: c.message,
                            user_id: self.get_client()?.uin().await.to_string(),
                        }
                        .into(),
                    )
                    .await,
                );
                self.database.insert_private_message(&s_private);
                Ok(Resps::success(respc.into()))
            }
            ty => Err(error_builder::unsupported_param(ty)),
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
                    self.get_client()?
                        .recall_friend_message(p.target_id, p.time as i64, p.seqs, p.rands)
                        .await
                        .map_err(error::rq_error)?;
                }
                SMessage::Group(g) => {
                    self.get_client()?
                        .recall_group_message(g.group_code, g.seqs, g.rands)
                        .await
                        .map_err(error::rq_error)?;
                }
            }
            Ok(Resps::empty_success())
        } else {
            Err(error::message_not_exist(&c.message_id))
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
            Err(error::message_not_exist(c.message_id))
        }
    }

    async fn get_self_info(&self) -> WQRespResult {
        Ok(Resps::success(
            UserInfoContent {
                user_id: self.get_client()?.uin().await.to_string(),
                nickname: self
                    .get_client()?
                    .account_info
                    .read()
                    .await
                    .nickname
                    .clone(),
                extra: ExtendedMap::default(),
            }
            .into(),
        ))
    }
    async fn get_user_info(&self, c: GetUserInfo) -> WQRespResult {
        let user_id: i64 = c.user_id.parse().map_err(|_| error::bad_param("user_id"))?;
        let info = self
            .get_client()?
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
            self.get_client()?
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
            .get_client()?
            .get_group_info(group_id)
            .await
            .map_err(error::rq_error)?
            .ok_or_else(|| error::group_not_exist(c.group_id))?;
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
            self.get_client()?
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
            .get_client()?
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
        self.get_client()?
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
        self.get_client()?
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
        self.get_client()?
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
        self.get_client()?
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
        self.get_client()?
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
        self.get_client()?
            .delete_friend(c.user_id.parse().map_err(|_| error::bad_param("user_id"))?)
            .await
            .map_err(error::rq_error)?;
        Ok(Resps::empty_success())
    }
    async fn get_new_friend_requests(&self) -> WQRespResult {
        Ok(Resps::success(
            ExtendedValue::List(
                self.get_client()?
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
    async fn set_join_group_request(&self, c: SetJoinGroup) -> WQRespResult {
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
        Ok(Resps::empty_success())
    }
    async fn get_join_group_requests(&self) -> WQRespResult {
        let joins = self
            .get_client()?
            .get_all_group_system_messages()
            .await
            .map_err(error::rq_error)?
            .join_group_requests;
        let mut v = vec![];
        for join in joins {
            v.push(
                new_event(
                    self.get_client()?,
                    Some(join.msg_time as f64),
                    WQRequestContent::JoinGroup {
                        sub_type: "".to_string(),
                        request_id: join.msg_seq,
                        user_id: join.req_uin.to_string(),
                        user_name: join.req_nick,
                        group_id: join.group_code.to_string(),
                        group_name: join.group_name,
                        message: join.message,
                        suspicious: join.suspicious,
                        invitor_id: join.invitor_uin.map(|i| i.to_string()),
                        invitor_name: join.invitor_nick,
                    }
                    .into(),
                )
                .await,
            )
        }
        Ok(Resps::success(RespContent::LatestEvents(v)))
    }
    async fn set_group_invite(&self, c: SetGroupInvited) -> WQRespResult {
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
        Ok(Resps::empty_success())
    }
    async fn get_group_invites(&self) -> WQRespResult {
        Ok(Resps::success(
            ExtendedValue::List(
                self.get_client()?
                    .get_all_group_system_messages()
                    .await
                    .map_err(error::rq_error)?
                    .self_invited
                    .into_iter()
                    .map(|i| {
                        extended_value!({
                            "request_id": i.msg_seq,
                            "group_id": i.group_code.to_string(),
                            "group_name": i.group_name,
                            "invitor_id": i.invitor_uin.to_string(),
                            "invitor_name": i.invitor_nick
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .into(),
        ))
    }
}
