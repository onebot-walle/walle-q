use rs_qq::client::{handler::Msg, msg::MsgElem};
use std::collections::HashMap;
use tracing::warn;
use walle_core::{Event, MessageContent, MessageSegment};

pub(crate) trait Parse<T> {
    fn parse(self) -> T;
}

pub(crate) trait Parser<X, Y> {
    fn parse(&self, input: X) -> Option<Y>;
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

impl Parser<Msg, Event> for walle_core::impls::OneBot {
    fn parse(&self, msg: Msg) -> Option<Event> {
        match msg {
            Msg::GroupMessage(group_message) => Some(
                self.new_event(
                    MessageContent::new_group_message_content(
                        group_message.elements.parse(),
                        group_message.sender.uin.to_string(),
                        group_message.group_code.to_string(),
                        HashMap::new(),
                    )
                    .into(),
                ),
            ),
            msg => {
                warn!("unsupported Msg: {:?}", msg);
                None
            }
        }
    }
}
