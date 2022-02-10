use async_trait::async_trait;
use rs_qq::client::handler::QEvent;
use rs_qq::msg::elem::{self, RQElem};
use rs_qq::msg::MessageChain;
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
    async fn parse(&self, msg: QEvent) -> Option<Event> {
        match msg {
            QEvent::Login(uin) => {
                info!("Walle-Q Login success with uin: {}", uin);
                *self.self_id.write().await = uin.to_string();
                None
            }
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
            msg => {
                warn!("unsupported Msg: {:?}", msg);
                None
            }
        }
    }
}
