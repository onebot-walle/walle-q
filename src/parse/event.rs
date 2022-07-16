use super::util::{
    new_event, new_group_audio_content, new_group_msg_content, new_group_temp_msg_content,
    new_private_audio_content, new_private_msg_content,
};
use crate::database::{Database, SVoice, WQDatabase};
use crate::model::{
    Delete, Disband, FriendPoke, GroupAdminSet, GroupAdminUnset, GroupInvite, GroupMemberBan,
    GroupNameUpdate, Join, JoinGroup, Kick, Leave, NewFriend, Recall, UserName, WalleQ, QQ,
};

use ricq::client::handler::QEvent;
use ricq::structs::GroupMemberPermission;
use tracing::{info, warn};
use walle_core::event::{
    Event, FriendDecrease, FriendIncrease, GroupMemberDecrease, GroupMemberIncrease,
    GroupMessageDelete, Notice, PrivateMessageDelete, Request,
};

pub(crate) async fn qevent2event(event: QEvent, wqdb: &WQDatabase) -> Option<Event> {
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
                (
                    Notice {},
                    PrivateMessageDelete {
                        message_id: e.inner.msg_seq.to_string(),
                        user_id: e.inner.friend_uin.to_string(),
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::NewFriend(e) => Some(
            new_event(
                &e.client,
                None,
                (
                    Notice {},
                    FriendIncrease {
                        user_id: e.inner.uin.to_string(),
                    },
                    (),
                    UserName {
                        user_name: e.inner.nick,
                    },
                    WalleQ {},
                ),
            )
            .await,
        ),

        // group
        QEvent::NewMember(e) => Some(
            new_event(
                &e.client,
                None,
                (
                    Notice {},
                    GroupMemberIncrease {
                        group_id: e.inner.group_code.to_string(),
                        user_id: e.inner.member_uin.to_string(),
                        operator_id: "".to_string(), //todo
                    },
                    Join {},
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::GroupLeave(e) => Some(if e.inner.operator_uin.is_some() {
            new_event(
                &e.client,
                None,
                (
                    Notice {},
                    GroupMemberDecrease {
                        group_id: e.inner.group_code.to_string(),
                        user_id: e.inner.member_uin.to_string(),
                        operator_id: e
                            .inner
                            .operator_uin
                            .clone()
                            .unwrap_or(e.inner.member_uin)
                            .to_string(),
                    },
                    Kick {},
                    QQ {},
                    WalleQ {},
                ),
            )
            .await
        } else {
            new_event(
                &e.client,
                None,
                (
                    Notice {},
                    GroupMemberDecrease {
                        group_id: e.inner.group_code.to_string(),
                        user_id: e.inner.member_uin.to_string(),
                        operator_id: e
                            .inner
                            .operator_uin
                            .clone()
                            .unwrap_or(e.inner.member_uin)
                            .to_string(),
                    },
                    Leave {},
                    QQ {},
                    WalleQ {},
                ),
            )
            .await
        }),
        QEvent::GroupMute(e) => Some(
            new_event(
                &e.client,
                None,
                (
                    Notice {},
                    GroupMemberBan {
                        group_id: e.inner.group_code.to_string(),
                        user_id: e.inner.target_uin.to_string(),
                        operator_id: e.inner.operator_uin.to_string(),
                        duration: e.inner.duration.as_secs() as i64,
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::GroupMessageRecall(e) => Some(if e.inner.author_uin == e.inner.operator_uin {
            new_event(
                &e.client,
                Some(e.inner.time as f64),
                (
                    Notice {},
                    GroupMessageDelete {
                        message_id: e.inner.msg_seq.to_string(), //todo
                        group_id: e.inner.group_code.to_string(),
                        user_id: e.inner.author_uin.to_string(),
                        operator_id: e.inner.operator_uin.to_string(),
                    },
                    Recall {},
                    QQ {},
                    WalleQ {},
                ),
            )
            .await
        } else {
            new_event(
                &e.client,
                Some(e.inner.time as f64),
                (
                    Notice {},
                    GroupMessageDelete {
                        message_id: e.inner.msg_seq.to_string(), //todo
                        group_id: e.inner.group_code.to_string(),
                        user_id: e.inner.author_uin.to_string(),
                        operator_id: e.inner.operator_uin.to_string(),
                    },
                    Delete {},
                    QQ {},
                    WalleQ {},
                ),
            )
            .await
        }),
        QEvent::MemberPermissionChange(e) => {
            match e.inner.new_permission {
                GroupMemberPermission::Administrator => Some(
                    new_event(
                        &e.client,
                        None,
                        (
                            Notice {},
                            GroupAdminSet {
                                group_id: e.inner.group_code.to_string(),
                                user_id: e.inner.member_uin.to_string(),
                                operator_id: "".to_string(), //todo
                            },
                            (),
                            QQ {},
                            WalleQ {},
                        ),
                    )
                    .await,
                ),
                GroupMemberPermission::Member => Some(
                    new_event(
                        &e.client,
                        None,
                        (
                            Notice {},
                            GroupAdminUnset {
                                group_id: e.inner.group_code.to_string(),
                                user_id: e.inner.member_uin.to_string(),
                                operator_id: "".to_string(), //todo
                            },
                            (),
                            QQ {},
                            WalleQ {},
                        ),
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
                (
                    Request {},
                    NewFriend {
                        request_id: fre.inner.msg_seq,
                        user_id: fre.inner.req_uin.to_string(),
                        user_name: fre.inner.req_nick,
                        message: fre.inner.message,
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::GroupRequest(gre) => Some(
            new_event(
                &gre.client,
                Some(gre.inner.msg_time as f64),
                (
                    Request {},
                    JoinGroup {
                        request_id: gre.inner.msg_seq,
                        user_id: gre.inner.req_uin.to_string(),
                        user_name: gre.inner.req_nick,
                        group_id: gre.inner.group_code.to_string(),
                        group_name: gre.inner.group_name,
                        message: gre.inner.message,
                        suspicious: gre.inner.suspicious,
                        invitor_id: gre.inner.invitor_uin.map(|i| i.to_string()),
                        invitor_name: gre.inner.invitor_nick,
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::SelfInvited(i) => Some(
            new_event(
                &i.client,
                Some(i.inner.msg_seq as f64),
                (
                    Request {},
                    GroupInvite {
                        request_id: i.inner.msg_seq,
                        group_id: i.inner.group_code.to_string(),
                        group_name: i.inner.group_name,
                        invitor_id: i.inner.invitor_uin.to_string(),
                        invitor_name: i.inner.invitor_nick,
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::GroupDisband(d) => Some(
            new_event(
                &d.client,
                None,
                (
                    Notice {},
                    GroupMemberDecrease {
                        group_id: d.inner.group_code.to_string(),
                        user_id: d.client.uin().await.to_string(),
                        operator_id: d.inner.operator_uin.to_string(),
                    },
                    Disband {},
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::GroupAudioMessage(gam) => {
            let message = vec![walle_core::segment::Voice {
                file_id: gam.inner.audio.0.hex_voice_id(),
            }
            .into()];
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
            let message = vec![walle_core::segment::Voice {
                file_id: fam.inner.audio.0.hex_voice_id(),
            }
            .into()];
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
                (
                    Notice {},
                    FriendPoke {
                        user_id: p.inner.sender.to_string(),
                        receiver_id: p.inner.receiver.to_string(),
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::GroupNameUpdate(g) => Some(
            new_event(
                &g.client,
                None,
                (
                    Notice {},
                    GroupNameUpdate {
                        group_id: g.inner.group_code.to_string(),
                        group_name: g.inner.group_name,
                        operator_id: g.inner.operator_uin.to_string(),
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
            )
            .await,
        ),
        QEvent::DeleteFriend(d) => Some(
            new_event(
                &d.client,
                None,
                (
                    Notice {},
                    FriendDecrease {
                        user_id: d.inner.uin.to_string(),
                    },
                    (),
                    QQ {},
                    WalleQ {},
                ),
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
