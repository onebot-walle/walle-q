use super::util::{
    new_event, new_group_audio_content, new_group_msg_content, new_group_temp_msg_content,
    new_private_audio_content, new_private_msg_content,
};
use crate::database::{Database, SVoice, WQDatabase};
use crate::extra::{WQEvent, WQExtraNoticeContent, WQRequestContent};

use ricq::client::handler::QEvent;
use ricq::structs::GroupMemberPermission;
use tracing::{info, warn};
use walle_core::prelude::*;

pub(crate) async fn qevent2event(event: QEvent, wqdb: &WQDatabase) -> Option<WQEvent> {
    match event {
        // meta
        QEvent::Login(uin) => {
            info!(
                target: crate::WALLE_Q,
                "Walle-Q Login success with uin: {}", uin
            );
            None
        }

        // message
        QEvent::FriendMessage(pme) => {
            let message = super::msg_chain2msg_seg_vec(pme.inner.elements.clone(), wqdb);
            let event = new_event(
                &pme.client,
                Some(pme.inner.time as f64),
                new_private_msg_content(pme.inner, message),
            )
            .await;
            wqdb.insert_message(&event);
            Some(event)
        }
        QEvent::GroupMessage(gme) => {
            let message = super::msg_chain2msg_seg_vec(gme.inner.elements.clone(), wqdb);
            let event = new_event(
                &gme.client,
                Some(gme.inner.time as f64),
                new_group_msg_content(gme.inner, message),
            )
            .await;
            wqdb.insert_message(&event);
            Some(event)
        }
        QEvent::GroupTempMessage(gtme) => {
            let message = super::msg_chain2msg_seg_vec(gtme.inner.elements.clone(), wqdb);
            let event = new_event(
                &gtme.client,
                Some(gtme.inner.time as f64),
                new_group_temp_msg_content(gtme.inner, message),
            )
            .await;
            wqdb.insert_message(&event);
            Some(event)
        }

        // notice
        // friend
        QEvent::FriendMessageRecall(e) => Some(
            new_event(
                &e.client,
                Some(e.inner.time as f64),
                NoticeContent::PrivateMessageDelete {
                    sub_type: "".to_string(),
                    message_id: e.inner.msg_seq.to_string(),
                    user_id: e.inner.friend_uin.to_string(),
                    extra: ExtendedMap::default(),
                }
                .into(),
            )
            .await,
        ),
        QEvent::NewFriend(e) => Some(
            new_event(
                &e.client,
                None,
                NoticeContent::FriendIncrease {
                    sub_type: "".to_string(),
                    user_id: e.inner.uin.to_string(),
                    extra: extended_map! {
                        "user_name": e.inner.nick,
                    },
                }
                .into(),
            )
            .await,
        ),

        // group
        QEvent::NewMember(e) => Some(
            new_event(
                &e.client,
                None,
                NoticeContent::GroupMemberIncrease {
                    sub_type: "join".to_string(),
                    group_id: e.inner.group_code.to_string(),
                    user_id: e.inner.member_uin.to_string(),
                    operator_id: "".to_string(),
                    extra: ExtendedMap::default(),
                }
                .into(),
            )
            .await,
        ),
        QEvent::GroupLeave(e) => Some(
            new_event(
                &e.client,
                None,
                NoticeContent::GroupMemberDecrease {
                    sub_type: if e.inner.operator_uin.is_none() {
                        "leave".to_string()
                    } else {
                        "kick".to_string()
                    },
                    group_id: e.inner.group_code.to_string(),
                    user_id: e.inner.member_uin.to_string(),
                    operator_id: if let Some(op) = e.inner.operator_uin {
                        op.to_string()
                    } else {
                        e.inner.member_uin.to_string()
                    },
                    extra: ExtendedMap::default(),
                }
                .into(),
            )
            .await,
        ),
        QEvent::GroupMute(e) => Some(
            new_event(
                &e.client,
                None,
                NoticeContent::GroupMemberBan {
                    sub_type: "".to_string(),
                    group_id: e.inner.group_code.to_string(),
                    user_id: e.inner.target_uin.to_string(),
                    operator_id: e.inner.operator_uin.to_string(),
                    extra: extended_map! {
                        "duration": e.inner.duration.as_secs() as i64,
                    },
                }
                .into(),
            )
            .await,
        ),
        QEvent::GroupMessageRecall(e) => Some(
            new_event(
                &e.client,
                Some(e.inner.time as f64),
                NoticeContent::GroupMessageDelete {
                    sub_type: if e.inner.author_uin == e.inner.operator_uin {
                        "recall".to_string()
                    } else {
                        "delete".to_string()
                    },
                    message_id: e.inner.msg_seq.to_string(),
                    group_id: e.inner.group_code.to_string(),
                    user_id: e.inner.author_uin.to_string(),
                    operator_id: e.inner.operator_uin.to_string(),
                    extra: ExtendedMap::default(),
                }
                .into(),
            )
            .await,
        ),
        QEvent::MemberPermissionChange(e) => {
            match e.inner.new_permission {
                GroupMemberPermission::Administrator => Some(
                    new_event(
                        &e.client,
                        None,
                        NoticeContent::GroupAdminSet {
                            sub_type: "".to_string(),
                            group_id: e.inner.group_code.to_string(),
                            user_id: e.inner.member_uin.to_string(),
                            operator_id: "".to_string(), //todo
                            extra: ExtendedMap::default(),
                        }
                        .into(),
                    )
                    .await,
                ),
                GroupMemberPermission::Member => Some(
                    new_event(
                        &e.client,
                        None,
                        NoticeContent::GroupAdminUnset {
                            sub_type: "".to_string(),
                            group_id: e.inner.group_code.to_string(),
                            user_id: e.inner.member_uin.to_string(),
                            operator_id: "".to_string(), //todo
                            extra: ExtendedMap::default(),
                        }
                        .into(),
                    )
                    .await,
                ),
                _ => None,
            }
        }
        QEvent::NewFriendRequest(fre) => Some(
            new_event(
                &fre.client,
                None,
                WQRequestContent::NewFriend {
                    sub_type: "".to_string(),
                    request_id: fre.inner.msg_seq,
                    user_id: fre.inner.req_uin.to_string(),
                    user_name: fre.inner.req_nick,
                    message: fre.inner.message,
                }
                .into(),
            )
            .await,
        ),
        QEvent::GroupRequest(gre) => Some(
            new_event(
                &gre.client,
                Some(gre.inner.msg_time as f64),
                WQRequestContent::JoinGroup {
                    sub_type: "".to_string(),
                    request_id: gre.inner.msg_seq,
                    user_id: gre.inner.req_uin.to_string(),
                    user_name: gre.inner.req_nick,
                    group_id: gre.inner.group_code.to_string(),
                    group_name: gre.inner.group_name,
                    message: gre.inner.message,
                    suspicious: gre.inner.suspicious,
                    invitor_id: gre.inner.invitor_uin.map(|i| i.to_string()),
                    invitor_name: gre.inner.invitor_nick,
                }
                .into(),
            )
            .await,
        ),
        QEvent::SelfInvited(i) => Some(
            new_event(
                &i.client,
                Some(i.inner.msg_seq as f64),
                WQRequestContent::GroupInvited {
                    sub_type: "".to_string(),
                    request_id: i.inner.msg_seq,
                    group_id: i.inner.group_code.to_string(),
                    group_name: i.inner.group_name,
                    invitor_id: i.inner.invitor_uin.to_string(),
                    invitor_name: i.inner.invitor_nick,
                }
                .into(),
            )
            .await,
        ),
        QEvent::GroupDisband(d) => Some(
            new_event(
                &d.client,
                None,
                NoticeContent::GroupMemberDecrease {
                    sub_type: "disband".to_string(),
                    group_id: d.inner.group_code.to_string(),
                    user_id: d.client.uin().await.to_string(),
                    operator_id: d.inner.operator_uin.to_string(),
                    extra: extended_map! {},
                }
                .into(),
            )
            .await,
        ),
        QEvent::GroupAudioMessage(gam) => {
            let message = vec![MessageSegment::audio(gam.inner.audio.0.hex_voice_id())];
            wqdb.insert_voice(&gam.inner.audio.0);
            let event = new_event(
                &gam.client,
                Some(gam.inner.time as f64),
                new_group_audio_content(gam.inner, message),
            )
            .await;
            wqdb.insert_message(&event);
            Some(event)
        }
        QEvent::FriendAudioMessage(fam) => {
            let message = vec![MessageSegment::audio(fam.inner.audio.0.hex_voice_id())];
            wqdb.insert_voice(&fam.inner.audio.0);
            let event = new_event(
                &fam.client,
                Some(fam.inner.time as f64),
                new_private_audio_content(fam.inner, message),
            )
            .await;
            wqdb.insert_message(&event);
            Some(event)
        }
        QEvent::FriendPoke(p) => Some(
            new_event(
                &p.client,
                None,
                WQExtraNoticeContent::FriendPock {
                    sub_type: "".to_string(),
                    user_id: p.inner.sender.to_string(),
                    receiver_id: p.inner.receiver.to_string(),
                }
                .into(),
            )
            .await,
        ),
        QEvent::GroupNameUpdate(g) => Some(
            new_event(
                &g.client,
                None,
                WQExtraNoticeContent::GroupNameUpdate {
                    sub_type: "".to_string(),
                    group_id: g.inner.group_code.to_string(),
                    group_name: g.inner.group_name,
                    operator_id: g.inner.operator_uin.to_string(),
                }
                .into(),
            )
            .await,
        ),
        QEvent::DeleteFriend(d) => Some(
            new_event(
                &d.client,
                None,
                NoticeContent::FriendDecrease {
                    sub_type: "".to_string(),
                    user_id: d.inner.uin.to_string(),
                    extra: extended_map! {},
                }
                .into(),
            )
            .await,
        ),
        QEvent::KickedOffline(_) => {
            warn!(target: crate::WALLE_Q, "Kicked Off 从其他客户端强制下线");
            None
        }
        QEvent::MSFOffline(_) => {
            warn!(target: crate::WALLE_Q, "MSF offline 服务器强制下线");
            None
        } // event => {
          //     warn!("unsupported event: {:?}", event);
          //     None
          // }
    }
}
