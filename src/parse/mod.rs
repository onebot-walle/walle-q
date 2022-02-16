pub mod v11;

use crate::database::Database;

use rs_qq::client::handler::QEvent;
use rs_qq::msg::elem::{self, RQElem};
use rs_qq::msg::MessageChain;
use rs_qq::structs::GroupMemberPermission;
use tracing::{debug, info, warn};
use walle_core::{Event, ExtendedMap, MessageContent, MessageSegment, NoticeContent};

pub fn rq_elem2msg_seg(elem: RQElem) -> Option<MessageSegment> {
    match elem {
        RQElem::Text(text) => Some(MessageSegment::text(text.content)),
        RQElem::At(elem::At { target: 0, .. }) => Some(MessageSegment::mention_all()),
        RQElem::At(at) => Some(MessageSegment::mention(at.target.to_string())),
        RQElem::Face(face) => Some(MessageSegment::Custom {
            ty: "face".to_owned(),
            data: [("file".to_string(), face.name.into())].into(),
        }),
        RQElem::MarketFace(face) => Some(MessageSegment::text(face.name)),
        RQElem::Dice(d) => Some(MessageSegment::Custom {
            ty: "dice".to_owned(),
            data: [("value".to_string(), (d.value as i64).into())].into(),
        }),
        RQElem::FingerGuessing(f) => Some(MessageSegment::Custom {
            ty: "rps".to_owned(),
            data: [(
                "value".to_string(),
                {
                    match f {
                        elem::FingerGuessing::Rock => 0,
                        elem::FingerGuessing::Scissors => 1,
                        elem::FingerGuessing::Paper => 2,
                    }
                }
                .into(),
            )]
            .into(),
        }),
        RQElem::LightApp(l) => Some(MessageSegment::Custom {
            ty: "json".to_owned(),
            data: [("data".to_string(), l.content.into())].into(),
        }),
        RQElem::FriendImage(i) => Some(MessageSegment::Image {
            file_id: i.image_id,
            extend: [("url".to_string(), i.url.into())].into(),
        }),
        RQElem::GroupImage(i) => Some(MessageSegment::Image {
            file_id: i.image_id,
            extend: [("url".to_string(), i.url.into())].into(),
        }),
        elem => {
            debug!("unsupported MsgElem: {:?}", elem);
            None
        }
    }
}

pub fn msg_chain2msg_seg_vec(chain: MessageChain) -> Vec<MessageSegment> {
    chain.into_iter().filter_map(rq_elem2msg_seg).collect()
}

pub fn msg_seg_vec2msg_chain(v: Vec<MessageSegment>) -> MessageChain {
    let mut chain = MessageChain::default();
    for msg_seg in v {
        match msg_seg {
            MessageSegment::Text { text, .. } => chain.push(elem::Text { content: text }),
            MessageSegment::Mention { user_id, .. } => {
                if let Ok(target) = user_id.parse() {
                    chain.push(elem::At {
                        display: user_id.to_string(),
                        target,
                    })
                }
            }
            MessageSegment::MentionAll { .. } => chain.push(elem::At {
                display: "all".to_string(),
                target: 0,
            }),
            seg => {
                warn!("unsupported MessageSegment: {:?}", seg);
                chain.push(elem::Text {
                    content: "unsupported MessageSegment".to_string(),
                })
            }
        }
    }
    chain
}

pub async fn qevent2event(ob: &walle_core::impls::OneBot, event: QEvent) -> Option<Event> {
    match event {
        // meta
        QEvent::TcpConnect | QEvent::TcpDisconnect => None,
        QEvent::Login(uin) => {
            *ob.self_id.write().await = uin.to_string();
            ob.set_online(true);
            info!("Walle-Q Login success with uin: {}", uin);
            None
        }

        // message
        QEvent::PrivateMessage(pme) => {
            let event = ob
                .new_event(
                    MessageContent::new_private_message_content(
                        msg_chain2msg_seg_vec(pme.message.elements),
                        pme.message.seqs[0].to_string(),
                        pme.message.from_uin.to_string(),
                        ExtendedMap::default(),
                    )
                    .into(),
                    pme.message.time as f64,
                )
                .await;
            crate::SLED_DB.insert_message_event(&event);
            Some(event)
        }
        QEvent::GroupMessage(gme) => {
            let event = ob
                .new_event(
                    MessageContent::new_group_message_content(
                        msg_chain2msg_seg_vec(gme.message.elements),
                        gme.message.seqs[0].to_string(),
                        gme.message.from_uin.to_string(),
                        gme.message.group_code.to_string(),
                        ExtendedMap::default(),
                    )
                    .into(),
                    gme.message.time as f64,
                )
                .await;
            crate::SLED_DB.insert_message_event(&event);
            Some(event)
        }
        QEvent::SelfGroupMessage(e) => {
            info!("SelfGroupMessage: {:?}", e);
            None
        }

        // notice
        // friend
        QEvent::FriendMessageRecall(e) => Some(
            ob.new_event(
                NoticeContent::PrivateMessageDelete {
                    sub_type: "".to_string(),
                    message_id: e.recall.msg_seq.to_string(),
                    user_id: e.recall.friend_uin.to_string(),
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
                }
                .into(),
                walle_core::timestamp_nano_f64(),
            )
            .await,
        ),
        QEvent::GroupLeave(e) => Some(
            ob.new_event(
                NoticeContent::GroupMemberDecrease {
                    sub_type: if e.leave.operator_uin.is_some() {
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
