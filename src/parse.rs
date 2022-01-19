use async_trait::async_trait;
use rs_qq::client::handler::QEvent;
use rs_qq::engine::*;
use std::collections::HashMap;
use tracing::{info, warn};
use walle_core::{Event, MessageContent, MessageSegment};

pub(crate) trait Parse<T> {
    fn parse(self) -> T;
}

#[async_trait]
pub(crate) trait Parser<X, Y> {
    async fn parse(&self, input: X) -> Option<Y>;
}

impl Parse<MessageSegment> for MsgElem {
    fn parse(self) -> MessageSegment {
        match self {
            Self::Text { content } => MessageSegment::Text {
                text: content,
                extend: HashMap::new(),
            },
            elem => {
                warn!("unsupported MsgElem: {:?}", elem);
                MessageSegment::Text {
                    text: "unsupported MsgElem".to_string(),
                    extend: HashMap::new(),
                }
            }
        }
    }
}

impl Parse<Vec<MessageSegment>> for Vec<MsgElem> {
    fn parse(self) -> Vec<MessageSegment> {
        self.into_iter().map(|elem| elem.parse()).collect()
    }
}

impl Parse<MsgElem> for MessageSegment {
    fn parse(self) -> MsgElem {
        match self {
            Self::Text { text, .. } => MsgElem::Text { content: text },
            msg_seg => {
                warn!("unsupported MessageSegment: {:?}", msg_seg);
                MsgElem::Text {
                    content: "unsupported MessageSegment".to_string(),
                }
            }
        }
    }
}

impl Parse<Vec<MsgElem>> for Vec<MessageSegment> {
    fn parse(self) -> Vec<MsgElem> {
        self.into_iter().map(|seg| seg.parse()).collect()
    }
}

#[async_trait]
impl Parser<QEvent, Event> for walle_core::impls::OneBot {
    async fn parse(&self, msg: QEvent) -> Option<Event> {
        match msg {
            QEvent::LoginEvent(uin) => {
                info!("Walle-Q Login success with uin: {}", uin);
                *self.self_id.write().await = uin.to_string();
                None
            }
            QEvent::GroupMessage(group_message) => Some(
                self.new_event(
                    MessageContent::new_group_message_content(
                        group_message.elements.parse(),
                        group_message.sender.uin.to_string(),
                        group_message.group_code.to_string(),
                        HashMap::new(),
                    )
                    .into(),
                )
                .await,
            ),
            QEvent::PrivateMessage(private) => Some(
                self.new_event(
                    MessageContent::new_private_message_content(
                        private.elements.parse(),
                        private.sender.uin.to_string(),
                        HashMap::new(),
                    )
                    .into(),
                )
                .await,
            ),
            msg => {
                warn!("unsupported Msg: {:?}", msg);
                None
            }
        }
    }
}
