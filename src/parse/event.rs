use super::util::{
    new_event, new_group_audio, new_group_msg, new_group_temp_msg, new_private_audio,
    new_private_msg,
};
use crate::database::{Database, SVoice, WQDatabase};
use crate::handler::Infos;
use crate::model::{
    Delete, Disband, FriendPoke, GroupAdminSet, GroupAdminUnset, GroupInvite, GroupMemberBan,
    GroupNameUpdate, GroupPoke, Join, JoinGroup, Kick, Leave, NewFriend, Recall, UserName, WalleQ,
    QQ,
};

use ricq::client::handler::QEvent;
use ricq::structs::GroupMemberPermission;
use tracing::{info, warn};
use walle_core::event::{
    Event, FriendDecrease, FriendIncrease, GroupMemberDecrease, GroupMemberIncrease,
    GroupMessageDelete, Meta, Notice, PrivateMessageDelete, Request, StatusUpdate,
};
use walle_core::structs::Selft;
use walle_core::{action::Action, resp::Resp, ActionHandler, EventHandler, GetStatus, OneBot};

pub(crate) async fn qevent2event<AH, EH>(
    event: QEvent,
    wqdb: &WQDatabase,
    infos: &Infos,
    self_id: i64,
    ob: &OneBot<AH, EH>,
) -> Option<Event>
where
    AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
    EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
{
    let selft = Selft {
        user_id: self_id.to_string(),
        platform: crate::PLATFORM.to_owned(),
    };
    Some(match event {
        // meta
        QEvent::Login(uin) => {
            info!(
                target: crate::WALLE_Q,
                "Walle-Q Login success with uin: {}", uin
            );
            new_event(
                None,
                (
                    Meta,
                    StatusUpdate {
                        status: ob.get_status().await,
                    },
                    (),
                    QQ,
                    WalleQ,
                ),
            )
        }

        // message
        QEvent::FriendMessage(pme) => {
            let message = super::msg_chain2msg_seg_vec(pme.inner.elements.clone(), wqdb);
            new_private_msg(pme.inner, message, selft, wqdb)
        }
        QEvent::GroupMessage(gme) => {
            let message = super::msg_chain2msg_seg_vec(gme.inner.elements.clone(), wqdb);
            new_group_msg(gme.inner, message, selft, wqdb)
        }
        QEvent::GroupTempMessage(gtme) => {
            let message = super::msg_chain2msg_seg_vec(gtme.inner.elements.clone(), wqdb);
            new_group_temp_msg(gtme.inner, message, selft, wqdb)
        }

        // notice
        // friend
        QEvent::FriendMessageRecall(e) => new_event(
            Some(e.inner.time as f64),
            (
                Notice { selft },
                PrivateMessageDelete {
                    message_id: e.inner.msg_seq.to_string(),
                    user_id: e.inner.friend_uin.to_string(),
                },
                (),
                QQ,
                WalleQ,
            ),
        ),

        QEvent::NewFriend(e) => new_event(
            None,
            (
                Notice { selft },
                FriendIncrease {
                    user_id: e.inner.uin.to_string(),
                },
                (),
                UserName {
                    user_name: e.inner.nick,
                },
                WalleQ,
            ),
        ),

        // group
        QEvent::NewMember(e) => {
            new_event(
                None,
                (
                    Notice { selft },
                    GroupMemberIncrease {
                        group_id: e.inner.group_code.to_string(),
                        user_id: e.inner.member_uin.to_string(),
                        operator_id: "".to_string(), //todo
                    },
                    Join {},
                    QQ,
                    WalleQ,
                ),
            )
        }
        QEvent::GroupLeave(e) => {
            if e.inner.operator_uin.is_some() {
                new_event(
                    None,
                    (
                        Notice { selft },
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
                        QQ,
                        WalleQ,
                    ),
                )
            } else {
                new_event(
                    None,
                    (
                        Notice { selft },
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
                        QQ,
                        WalleQ,
                    ),
                )
            }
        }
        QEvent::GroupMute(e) => new_event(
            None,
            (
                Notice { selft },
                GroupMemberBan {
                    group_id: e.inner.group_code.to_string(),
                    user_id: e.inner.target_uin.to_string(),
                    operator_id: e.inner.operator_uin.to_string(),
                    duration: e.inner.duration.as_secs() as i64,
                },
                (),
                QQ,
                WalleQ,
            ),
        ),
        QEvent::GroupMessageRecall(e) => {
            if e.inner.author_uin == e.inner.operator_uin {
                new_event(
                    Some(e.inner.time as f64),
                    (
                        Notice { selft },
                        GroupMessageDelete {
                            message_id: e.inner.msg_seq.to_string(), //todo
                            group_id: e.inner.group_code.to_string(),
                            user_id: e.inner.author_uin.to_string(),
                            operator_id: e.inner.operator_uin.to_string(),
                        },
                        Recall {},
                        QQ,
                        WalleQ,
                    ),
                )
            } else {
                new_event(
                    Some(e.inner.time as f64),
                    (
                        Notice { selft },
                        GroupMessageDelete {
                            message_id: e.inner.msg_seq.to_string(), //todo
                            group_id: e.inner.group_code.to_string(),
                            user_id: e.inner.author_uin.to_string(),
                            operator_id: e.inner.operator_uin.to_string(),
                        },
                        Delete {},
                        QQ,
                        WalleQ,
                    ),
                )
            }
        }
        QEvent::MemberPermissionChange(e) => {
            match e.inner.new_permission {
                GroupMemberPermission::Administrator => {
                    if e.inner.member_uin == self_id {
                        if let Some((_, info)) = infos
                            .groups
                            .remove(&e.inner.group_code)
                            .or_else(|| infos.owned_groups.remove(&e.inner.group_code))
                        {
                            infos.admined_groups.insert(e.inner.group_code, info);
                        }
                    }
                    new_event(
                        None,
                        (
                            Notice { selft },
                            GroupAdminSet {
                                group_id: e.inner.group_code.to_string(),
                                user_id: e.inner.member_uin.to_string(),
                                operator_id: "".to_string(), //todo
                            },
                            (),
                            QQ,
                            WalleQ,
                        ),
                    )
                }
                GroupMemberPermission::Member => {
                    if e.inner.member_uin == self_id {
                        if let Some((_, info)) = infos
                            .admined_groups
                            .remove(&e.inner.group_code)
                            .or_else(|| infos.owned_groups.remove(&e.inner.group_code))
                        {
                            infos.groups.insert(e.inner.group_code, info);
                        }
                    }
                    new_event(
                        None,
                        (
                            Notice { selft },
                            GroupAdminUnset {
                                group_id: e.inner.group_code.to_string(),
                                user_id: e.inner.member_uin.to_string(),
                                operator_id: "".to_string(), //todo
                            },
                            (),
                            QQ,
                            WalleQ,
                        ),
                    )
                }
                GroupMemberPermission::Owner => {
                    if e.inner.member_uin == self_id {
                        if let Some((_, info)) = infos
                            .groups
                            .remove(&e.inner.group_code)
                            .or_else(|| infos.admined_groups.remove(&e.inner.group_code))
                        {
                            infos.owned_groups.insert(e.inner.group_code, info);
                        }
                    }
                    new_event(
                        None,
                        (
                            Notice { selft },
                            GroupAdminSet {
                                group_id: e.inner.group_code.to_string(),
                                user_id: e.inner.member_uin.to_string(),
                                operator_id: "".to_string(), //todo
                            },
                            (),
                            QQ,
                            WalleQ,
                        ),
                    ) //todo
                }
            }
        }
        QEvent::NewFriendRequest(fre) => new_event(
            None,
            (
                Request { selft },
                NewFriend {
                    request_id: fre.inner.msg_seq,
                    user_id: fre.inner.req_uin.to_string(),
                    user_name: fre.inner.req_nick,
                    message: fre.inner.message,
                },
                (),
                QQ,
                WalleQ,
            ),
        ),
        QEvent::GroupRequest(gre) => new_event(
            Some(gre.inner.msg_time as f64),
            (
                Request { selft },
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
                QQ,
                WalleQ,
            ),
        ),

        QEvent::SelfInvited(i) => new_event(
            Some(i.inner.msg_seq as f64),
            (
                Request { selft },
                GroupInvite {
                    request_id: i.inner.msg_seq,
                    group_id: i.inner.group_code.to_string(),
                    group_name: i.inner.group_name,
                    invitor_id: i.inner.invitor_uin.to_string(),
                    invitor_name: i.inner.invitor_nick,
                },
                (),
                QQ,
                WalleQ,
            ),
        ),

        QEvent::GroupDisband(d) => new_event(
            None,
            (
                Notice { selft },
                GroupMemberDecrease {
                    group_id: d.inner.group_code.to_string(),
                    user_id: d.client.uin().await.to_string(),
                    operator_id: d.inner.operator_uin.to_string(),
                },
                Disband {},
                QQ,
                WalleQ,
            ),
        ),

        QEvent::GroupAudioMessage(gam) => {
            let message = vec![walle_core::segment::Voice {
                file_id: gam.inner.audio.0.hex_voice_id(),
            }
            .into()];
            wqdb.insert_voice(&gam.inner.audio.0);
            let event = new_group_audio(gam.inner, message, selft, wqdb);
            event
        }
        QEvent::FriendAudioMessage(fam) => {
            let message = vec![walle_core::segment::Voice {
                file_id: fam.inner.audio.0.hex_voice_id(),
            }
            .into()];
            wqdb.insert_voice(&fam.inner.audio.0);
            new_private_audio(fam.inner, message, selft, wqdb)
        }
        QEvent::FriendPoke(p) => new_event(
            None,
            (
                Notice { selft },
                FriendPoke {
                    user_id: p.inner.sender.to_string(),
                    receiver_id: p.inner.receiver.to_string(),
                },
                (),
                QQ,
                WalleQ,
            ),
        ),
        QEvent::GroupPoke(p) => new_event(
            None,
            (
                Notice { selft },
                GroupPoke {
                    group_id: p.inner.group_code.to_string(),
                    user_id: p.inner.sender.to_string(),
                    receiver_id: p.inner.receiver.to_string(),
                },
                (),
                QQ,
                WalleQ,
            ),
        ),

        QEvent::GroupNameUpdate(g) => {
            if let Some(mut group_info) = infos
                .groups
                .get_mut(&g.inner.group_code)
                .or_else(|| infos.admined_groups.get_mut(&g.inner.group_code))
                .or_else(|| infos.owned_groups.get_mut(&g.inner.group_code))
            {
                group_info.group_name = g.inner.group_name.clone();
            }
            new_event(
                None,
                (
                    Notice { selft },
                    GroupNameUpdate {
                        group_id: g.inner.group_code.to_string(),
                        group_name: g.inner.group_name,
                        operator_id: g.inner.operator_uin.to_string(),
                    },
                    (),
                    QQ,
                    WalleQ,
                ),
            )
        }
        QEvent::DeleteFriend(d) => {
            infos.friends.remove(&d.inner.uin);
            new_event(
                None,
                (
                    Notice { selft },
                    FriendDecrease {
                        user_id: d.inner.uin.to_string(),
                    },
                    (),
                    QQ,
                    WalleQ,
                ),
            )
        }
        QEvent::KickedOffline(_) => {
            warn!(target: crate::WALLE_Q, "Kicked Off 从其他客户端强制下线");
            new_event(
                None,
                (
                    Meta,
                    StatusUpdate {
                        status: ob.get_status().await,
                    },
                    (),
                    QQ,
                    WalleQ,
                ),
            )
        }
        QEvent::MSFOffline(_) => {
            warn!(target: crate::WALLE_Q, "MSF offline 服务器强制下线");
            new_event(
                None,
                (
                    Meta,
                    StatusUpdate {
                        status: ob.get_status().await,
                    },
                    (),
                    QQ,
                    WalleQ,
                ),
            )
        }
        QEvent::ClientDisconnect(_) => {
            warn!(target: crate::WALLE_Q, "网络断线，自动重连。。。");
            return None;
        }
    })
}
