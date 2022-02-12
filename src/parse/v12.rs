use super::{Parse, Parser};
use async_trait::async_trait;
use rs_qq::client::handler::QEvent;
use rs_qq::msg::elem::{self, RQElem};
use rs_qq::msg::MessageChain;
use std::collections::HashMap;
use tracing::{info, warn};
use walle_core::{Event, MessageContent, MessageSegment, NoticeContent};

impl Parse<Option<MessageSegment>> for RQElem {
    fn parse(self) -> Option<MessageSegment> {
        match self {
            Self::Text(text) => Some(MessageSegment::text(text.content)),
            Self::Other(_) => None,
            Self::At(elem::At { target: 0, .. }) => Some(MessageSegment::mention_all()),
            Self::At(at) => Some(MessageSegment::mention(at.target.to_string())),
            elem => {
                warn!("unsupported MsgElem: {:?}", elem);
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
        match event {
            QEvent::TcpConnect | QEvent::TcpDisconnect => None,
            QEvent::Login(uin) => {
                *self.self_id.write().await = uin.to_string();
                self.set_online(true);
                info!("Walle-Q Login success with uin: {}", uin);
                None
            }

            QEvent::PrivateMessage(private) => Some(
                self.new_event(
                    MessageContent::new_private_message_content(
                        private.message.elements.parse(),
                        private.message.from_uin.to_string(),
                        HashMap::new(),
                    )
                    .into(),
                )
                .await,
            ),
            QEvent::GroupMessage(gme) => Some(
                self.new_event(
                    MessageContent::new_group_message_content(
                        gme.message.elements.parse(),
                        gme.message.from_uin.to_string(),
                        gme.message.group_code.to_string(),
                        HashMap::new(),
                    )
                    .into(),
                )
                .await,
            ),
            QEvent::SelfGroupMessage(e) => {
                info!("SelfGroupMessage: {:?}", e);
                None
            }

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
            event => {
                warn!("unsupported event: {:?}", event);
                None
            }
        }
    }
}
