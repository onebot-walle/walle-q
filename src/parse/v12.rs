use crate::database::Database;

use super::{Parse, Parser};
use async_trait::async_trait;
use rs_qq::client::handler::QEvent;
use rs_qq::msg::elem::{self, RQElem};
use rs_qq::msg::MessageChain;
use rs_qq::structs::GroupMemberPermission;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use walle_core::{Event, ExtendedMap, MessageContent, MessageSegment, NoticeContent};

impl Parse<Option<MessageSegment>> for RQElem {
    fn parse(self) -> Option<MessageSegment> {
        match self {
            Self::Text(text) => Some(MessageSegment::text(text.content)),
            Self::At(elem::At { target: 0, .. }) => Some(MessageSegment::mention_all()),
            Self::At(at) => Some(MessageSegment::mention(at.target.to_string())),
            Self::Face(face) => Some(MessageSegment::Custom {
                ty: "face".to_owned(),
                data: [("file".to_string(), face.name.into())].into(),
            }),
            Self::MarketFace(face) => Some(MessageSegment::text(face.name)),
            Self::Dice(d) => Some(MessageSegment::Custom {
                ty: "dice".to_owned(),
                data: [("value".to_string(), (d.value as i64).into())].into(),
            }),
            Self::FingerGuessing(f) => Some(MessageSegment::Custom {
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
            Self::LightApp(l) => Some(MessageSegment::Custom {
                ty: "json".to_owned(),
                data: [("data".to_string(), l.content.into())].into(),
            }),
            Self::FriendImage(i) => Some(MessageSegment::Image {
                file_id: i.image_id,
                extend: [("url".to_string(), i.url.into())].into(),
            }),
            Self::GroupImage(i) => Some(MessageSegment::Image {
                file_id: i.image_id,
                extend: [("url".to_string(), i.url.into())].into(),
            }),
            elem => {
                debug!("unsupported MsgElem: {:?}", elem);
                Some(MessageSegment::Text {
                    text: "unsupported MsgElem".to_string(),
                    extend: HashMap::new(),
                })
            }
        }
    }
}

impl Parse<Vec<MessageSegment>> for MessageChain {
    fn parse(self) -> Vec<MessageSegment> {
        self.into_iter().filter_map(|elem| elem.parse()).collect()
    }
}

impl Parse<MessageChain> for Vec<MessageSegment> {
    fn parse(self) -> MessageChain {
        let mut chain = MessageChain::default();
        for msg_seg in self {
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
}

#[async_trait]
impl Parser<QEvent, Event> for walle_core::impls::OneBot {
    async fn parse(&self, event: QEvent) -> Option<Event> {
        fn message_id_map(seqs: &Vec<i32>) -> ExtendedMap {
            [("qq.message_id".to_owned(), (seqs[0] as i64).into())].into()
        }

        match event {
            // meta
            QEvent::TcpConnect | QEvent::TcpDisconnect => None,
            QEvent::Login(uin) => {
                *self.self_id.write().await = uin.to_string();
                self.set_online(true);
                info!("Walle-Q Login success with uin: {}", uin);
                None
            }

            // message
            QEvent::PrivateMessage(pme) => {
                let event = self
                    .new_event(
                        MessageContent::new_private_message_content(
                            pme.message.elements.parse(),
                            pme.message.from_uin.to_string(),
                            message_id_map(&pme.message.seqs),
                        )
                        .into(),
                    )
                    .await;
                crate::SLED_DB.insert_event(pme.message.seqs[0], &event);
                Some(event)
            }
            QEvent::GroupMessage(gme) => {
                let event = self
                    .new_event(
                        MessageContent::new_group_message_content(
                            gme.message.elements.parse(),
                            gme.message.from_uin.to_string(),
                            gme.message.group_code.to_string(),
                            message_id_map(&gme.message.seqs),
                        )
                        .into(),
                    )
                    .await;
                crate::SLED_DB.insert_event(gme.message.seqs[0], &event);
                Some(event)
            }
            QEvent::SelfGroupMessage(e) => {
                info!("SelfGroupMessage: {:?}", e);
                None
            }

            // notice
            // friend
            QEvent::FriendMessageRecall(e) => Some(
                self.new_event(
                    NoticeContent::PrivateMessageDelete {
                        sub_type: "".to_string(),
                        message_id: e.recall.msg_seq.to_string(),
                        user_id: e.recall.friend_uin.to_string(),
                    }
                    .into(),
                )
                .await,
            ),
            QEvent::NewFriend(e) => Some(
                self.new_event(
                    NoticeContent::FriendIncrease {
                        sub_type: "".to_string(),
                        user_id: e.friend.uin.to_string(),
                    }
                    .into(),
                )
                .await,
            ),

            // group
            QEvent::NewMember(e) => Some(
                self.new_event(
                    NoticeContent::GroupMemberIncrease {
                        sub_type: "join".to_string(),
                        group_id: e.new_member.group_code.to_string(),
                        user_id: e.new_member.member_uin.to_string(),
                        operator_id: "".to_string(),
                    }
                    .into(),
                )
                .await,
            ),
            QEvent::GroupLeave(e) => Some(
                self.new_event(
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
                )
                .await,
            ),
            QEvent::GroupMute(e) => Some(
                self.new_event(
                    NoticeContent::GroupMemberBan {
                        sub_type: "".to_string(),
                        group_id: e.group_mute.group_code.to_string(),
                        user_id: e.group_mute.target_uin.to_string(),
                        operator_id: e.group_mute.operator_uin.to_string(),
                    }
                    .into(),
                )
                .await,
            ),
            QEvent::GroupMessageRecall(e) => Some(
                self.new_event(
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
                )
                .await,
            ),
            QEvent::MemberPermissionChange(e) => {
                let e = &e.change;
                match e.new_permission {
                    GroupMemberPermission::Administrator => Some(
                        self.new_event(
                            NoticeContent::GroupAdminSet {
                                sub_type: "".to_string(),
                                group_id: e.group_code.to_string(),
                                user_id: e.member_uin.to_string(),
                                operator_id: "".to_string(), //todo
                            }
                            .into(),
                        )
                        .await,
                    ),
                    GroupMemberPermission::Member => Some(
                        self.new_event(
                            NoticeContent::GroupAdminUnset {
                                sub_type: "".to_string(),
                                group_id: e.group_code.to_string(),
                                user_id: e.member_uin.to_string(),
                                operator_id: "".to_string(), //todo
                            }
                            .into(),
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
}
