use crate::database::{Database, SGroupMessage, SPrivateMessage, WQDatabase};
use crate::handler::OneBot;

use ricq::client::handler::QEvent;
use ricq::structs::GroupMemberPermission;
use tracing::{info, warn};
use walle_core::{ExtendedMap, MessageContent, NoticeContent, StandardEvent};

pub(crate) async fn qevent2event(
    ob: &OneBot,
    event: QEvent,
    wqdb: &WQDatabase,
) -> Option<StandardEvent> {
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
                        ExtendedMap::default(),
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
                        ExtendedMap::default(),
                    )
                    .into(),
                    gme.message.time as f64,
                )
                .await;
            let s_group = SGroupMessage::new(gme.message, event.clone());
            wqdb.insert_group_message(&s_group);
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
                    extra: ExtendedMap::default(),
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
                    extra: ExtendedMap::default(),
                }
                .into(),
                e.group_mute.time as f64,
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

        event => {
            warn!("unsupported event: {:?}", event);
            None
        }
    }
}
