use crate::database::{Database, SGroupMessage, SPrivateMessage, WQDatabase};
use crate::extra::{WQEvent, WQRequestContent};
use crate::OneBot;

use ricq::client::handler::QEvent;
use ricq::structs::GroupMemberPermission;
use tracing::{info, warn};
use walle_core::{extended_map, MessageAlt, MessageEventDetail};
use walle_core::{ExtendedMap, MessageContent, NoticeContent};

pub(crate) async fn qevent2event(ob: &OneBot, event: QEvent, wqdb: &WQDatabase) -> Option<WQEvent> {
    match event {
        // meta
        QEvent::Login(uin) => {
            *ob.self_id.write().await = uin.to_string();
            ob.set_online(true);
            info!(
                target: crate::WALLE_Q,
                "Walle-Q Login success with uin: {}", uin
            );
            None
        }

        // message
        QEvent::FriendMessage(pme) => {
            let message = super::msg_chain2msg_seg_vec(pme.message.elements.clone(), wqdb);
            let event = ob
                .new_event(
                    MessageContent::new_private_message_content(
                        message,
                        pme.message.seqs[0].to_string(),
                        pme.message.from_uin.to_string(),
                        extended_map! {
                            "user_name": pme.message.from_nick
                        },
                    )
                    .into(),
                    pme.message.time as f64,
                )
                .await;
            let s_private = SPrivateMessage::new(pme.message, event.clone());
            wqdb.insert_private_message(&s_private);
            Some(event)
        }
        QEvent::GroupMessage(gme) => {
            let message = super::msg_chain2msg_seg_vec(gme.message.elements.clone(), wqdb);
            let event = ob
                .new_event(
                    MessageContent::new_group_message_content(
                        message,
                        gme.message.seqs[0].to_string(),
                        gme.message.from_uin.to_string(),
                        gme.message.group_code.to_string(),
                        extended_map! {
                            "user_card": gme.message.group_card,
                            "group_name": gme.message.group_name,
                        },
                    )
                    .into(),
                    gme.message.time as f64,
                )
                .await;
            let s_group = SGroupMessage::new(gme.message, event.clone());
            wqdb.insert_group_message(&s_group);
            Some(event)
        }
        QEvent::GroupTempMessage(gtme) => {
            let message = super::msg_chain2msg_seg_vec(gtme.message.elements.clone(), wqdb);
            let event = ob
                .new_event(
                    MessageContent {
                        alt_message: message.alt(),
                        message,
                        message_id: gtme.message.seqs[0].to_string(),
                        user_id: gtme.message.from_uin.to_string(),
                        detail: MessageEventDetail::Private {
                            sub_type: "group_temp".to_owned(),
                            extra: extended_map! {
                                "group_id": gtme.message.group_code.to_string(),
                                "user_name": gtme.message.from_nick,
                            },
                        },
                    }
                    .into(),
                    gtme.message.time as f64,
                )
                .await;
            let s_private = SPrivateMessage::from_temp(gtme.message, event.clone());
            wqdb.insert_private_message(&s_private);
            Some(event)
        }

        // notice
        // friend
        QEvent::FriendMessageRecall(e) => Some(
            ob.new_event(
                NoticeContent::PrivateMessageDelete {
                    sub_type: "".to_string(),
                    message_id: e.recall.msg_seq.to_string(),
                    user_id: e.recall.friend_uin.to_string(),
                    extra: ExtendedMap::default(),
                }
                .into(),
                e.recall.time as f64,
            )
            .await,
        ),
        QEvent::NewFriend(e) => Some(
            ob.new_event(
                NoticeContent::FriendIncrease {
                    sub_type: "".to_string(),
                    user_id: e.friend.uin.to_string(),
                    extra: extended_map! {
                        "user_name": e.friend.nick,
                    },
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),

        // group
        QEvent::NewMember(e) => Some(
            ob.new_event(
                NoticeContent::GroupMemberIncrease {
                    sub_type: "join".to_string(),
                    group_id: e.new_member.group_code.to_string(),
                    user_id: e.new_member.member_uin.to_string(),
                    operator_id: "".to_string(),
                    extra: ExtendedMap::default(),
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),
        QEvent::GroupLeave(e) => Some(
            ob.new_event(
                NoticeContent::GroupMemberDecrease {
                    sub_type: if e.leave.operator_uin.is_none() {
                        "leave".to_string()
                    } else {
                        "kick".to_string()
                    },
                    group_id: e.leave.group_code.to_string(),
                    user_id: e.leave.member_uin.to_string(),
                    operator_id: if let Some(op) = e.leave.operator_uin {
                        op.to_string()
                    } else {
                        e.leave.member_uin.to_string()
                    },
                    extra: ExtendedMap::default(),
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),
        QEvent::GroupMute(e) => Some(
            ob.new_event(
                NoticeContent::GroupMemberBan {
                    sub_type: "".to_string(),
                    group_id: e.group_mute.group_code.to_string(),
                    user_id: e.group_mute.target_uin.to_string(),
                    operator_id: e.group_mute.operator_uin.to_string(),
                    extra: extended_map! {
                        "duration": e.group_mute.duration.as_secs() as i64,
                    },
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),
        QEvent::GroupMessageRecall(e) => Some(
            ob.new_event(
                NoticeContent::GroupMessageDelete {
                    sub_type: if e.recall.author_uin == e.recall.operator_uin {
                        "recall".to_string()
                    } else {
                        "delete".to_string()
                    },
                    message_id: e.recall.msg_seq.to_string(),
                    group_id: e.recall.group_code.to_string(),
                    user_id: e.recall.author_uin.to_string(),
                    operator_id: e.recall.operator_uin.to_string(),
                    extra: ExtendedMap::default(),
                }
                .into(),
                e.recall.time as f64,
            )
            .await,
        ),
        QEvent::MemberPermissionChange(e) => {
            let e = &e.change;
            match e.new_permission {
                GroupMemberPermission::Administrator => Some(
                    ob.new_event(
                        NoticeContent::GroupAdminSet {
                            sub_type: "".to_string(),
                            group_id: e.group_code.to_string(),
                            user_id: e.member_uin.to_string(),
                            operator_id: "".to_string(), //todo
                            extra: ExtendedMap::default(),
                        }
                        .into(),
                        walle_core::timestamp_nano_f64(),
                    )
                    .await,
                ),
                GroupMemberPermission::Member => Some(
                    ob.new_event(
                        NoticeContent::GroupAdminUnset {
                            sub_type: "".to_string(),
                            group_id: e.group_code.to_string(),
                            user_id: e.member_uin.to_string(),
                            operator_id: "".to_string(), //todo
                            extra: ExtendedMap::default(),
                        }
                        .into(),
                        walle_core::timestamp_nano_f64(),
                    )
                    .await,
                ),
                _ => None,
            }
        }
        QEvent::FriendRequest(fre) => Some(
            ob.new_event(
                WQRequestContent::NewFriend {
                    sub_type: "".to_string(),
                    request_id: fre.request.msg_seq,
                    user_id: fre.request.req_uin.to_string(),
                    user_name: fre.request.req_nick,
                    message: fre.request.message,
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),
        QEvent::GroupRequest(gre) => Some(
            ob.new_event(
                WQRequestContent::JoinGroup {
                    sub_type: "".to_string(),
                    request_id: gre.request.msg_seq,
                    user_id: gre.request.req_uin.to_string(),
                    user_name: gre.request.req_nick,
                    group_id: gre.request.group_code.to_string(),
                    group_name: gre.request.group_name,
                    message: gre.request.message,
                    suspicious: gre.request.suspicious,
                    invitor_id: gre.request.invitor_uin.map(|i| i.to_string()),
                    invitor_name: gre.request.invitor_nick,
                }
                .into(),
                gre.request.msg_time as f64,
            )
            .await,
        ),
        // QEvent::GroupAudioMessage(_) => todo!(),
        // QEvent::FriendAudioMessage(_) => todo!(),
        QEvent::SelfInvited(i) => Some(
            ob.new_event(
                WQRequestContent::GroupInvited {
                    sub_type: "".to_string(),
                    request_id: i.request.msg_seq,
                    group_id: i.request.group_code.to_string(),
                    group_name: i.request.group_name,
                    invitor_id: i.request.invitor_uin.to_string(),
                    invitor_name: i.request.invitor_nick,
                }
                .into(),
                i.request.msg_time as f64,
            )
            .await,
        ),
        QEvent::GroupDisband(d) => Some(
            ob.new_event(
                NoticeContent::GroupMemberDecrease {
                    sub_type: "disband".to_string(),
                    group_id: d.disband.group_code.to_string(),
                    user_id: ob.self_id().await,
                    operator_id: d.disband.operator_uin.to_string(),
                    extra: extended_map! {},
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),
        // QEvent::FriendPoke(_) => todo!(),
        // QEvent::GroupNameUpdate(_) => todo!(),
        QEvent::DeleteFriend(d) => Some(
            ob.new_event(
                NoticeContent::FriendDecrease {
                    sub_type: "".to_string(),
                    user_id: d.delete.uin.to_string(),
                    extra: extended_map! {},
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),
        QEvent::KickedOffline(_) => {
            warn!(target: crate::WALLE_Q, "Kicked Off 从其他客户端强制下线");
            ob.shutdown().await;
            None
        }
        QEvent::MSFOffline(_) => {
            warn!(target: crate::WALLE_Q, "MSF offline 服务器强制下线");
            ob.shutdown().await; // 求测试2333
            None
        }

        event => {
            warn!("unsupported event: {:?}", event);
            None
        }
    }
}
