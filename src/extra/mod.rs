mod action;
mod request;
pub mod segment;

pub use action::*;
pub use request::*;

use serde::{Deserialize, Serialize};
use walle_core::{
    BaseEvent, ColoredAlt, MessageContent, MessageEvent, MessageEventDetail, MetaContent,
    NoticeContent,
};

pub type WQEvent = BaseEvent<WQEventContent>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WQEventContent {
    Meta(MetaContent),
    Message(MessageContent<MessageEventDetail>),
    Notice(NoticeContent),
    Request(WQRequestContent),
}

impl From<MetaContent> for WQEventContent {
    fn from(meta: MetaContent) -> Self {
        WQEventContent::Meta(meta)
    }
}

impl From<MessageContent<MessageEventDetail>> for WQEventContent {
    fn from(message: MessageContent<MessageEventDetail>) -> Self {
        WQEventContent::Message(message)
    }
}

impl From<NoticeContent> for WQEventContent {
    fn from(notice: NoticeContent) -> Self {
        WQEventContent::Notice(notice)
    }
}

impl From<WQRequestContent> for WQEventContent {
    fn from(request: WQRequestContent) -> Self {
        WQEventContent::Request(request)
    }
}

pub(crate) trait ToMessageEvent {
    fn to_message_event(self) -> Option<MessageEvent>;
}

impl ToMessageEvent for WQEvent {
    fn to_message_event(self) -> Option<MessageEvent> {
        match self.content {
            WQEventContent::Message(message) => Some(MessageEvent {
                id: self.id,
                r#impl: self.r#impl,
                platform: self.platform,
                self_id: self.self_id,
                time: self.time,
                content: message,
            }),
            _ => None,
        }
    }
}

impl ColoredAlt for WQEventContent {
    fn colored_alt(&self) -> Option<String> {
        match self {
            WQEventContent::Message(message) => message.colored_alt(),
            WQEventContent::Notice(notice) => notice.colored_alt(),
            WQEventContent::Request(request) => request.colored_alt(),
            _ => None,
        }
    }
}
