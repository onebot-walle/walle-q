mod action;
mod message;
mod notice;
mod request;
pub mod segment;

pub use action::*;
use colored::Colorize;
pub use message::*;
pub use notice::*;
pub use request::*;

use serde::{Deserialize, Serialize};
use walle_core::event::{BaseEvent, MessageContent, MetaContent};
use walle_core::util::ColoredAlt;

pub type WQEvent = BaseEvent<WQEventContent>;
pub type WQMessageEvent = BaseEvent<MessageContent<WQMEDetail>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WQEventContent {
    Meta(MetaContent),
    Message(MessageContent<WQMEDetail>),
    Notice(WQNoticeContent),
    Request(WQRequestContent),
}

impl From<MetaContent> for WQEventContent {
    fn from(meta: MetaContent) -> Self {
        WQEventContent::Meta(meta)
    }
}

impl From<MessageContent<WQMEDetail>> for WQEventContent {
    fn from(message: MessageContent<WQMEDetail>) -> Self {
        WQEventContent::Message(message)
    }
}

impl From<WQNoticeContent> for WQEventContent {
    fn from(notice: WQNoticeContent) -> Self {
        WQEventContent::Notice(notice)
    }
}

impl From<WQRequestContent> for WQEventContent {
    fn from(request: WQRequestContent) -> Self {
        WQEventContent::Request(request)
    }
}

pub(crate) trait ToMessageEvent {
    fn to_message_event(self) -> Option<WQMessageEvent>;
}

impl ToMessageEvent for WQEvent {
    fn to_message_event(self) -> Option<WQMessageEvent> {
        match self.content {
            WQEventContent::Message(message) => Some(WQMessageEvent {
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
            WQEventContent::Message(message) => match &message.detail {
                WQMEDetail::Group { group_id, .. } => Some(format!(
                    "[{}] {} from {}",
                    group_id.bright_blue(),
                    message.alt_message,
                    message.user_id.bright_green()
                )),
                WQMEDetail::Private { .. } => Some(format!(
                    "[{}] {}",
                    message.user_id.bright_green(),
                    message.alt_message,
                )),
                WQMEDetail::GroupTemp { group_id, .. } => Some(format!(
                    "[{}] {} from {}",
                    message.user_id.bright_green(),
                    message.alt_message,
                    group_id.bright_blue()
                )),
            },
            WQEventContent::Notice(notice) => notice.colored_alt(),
            WQEventContent::Request(request) => request.colored_alt(),
            _ => None,
        }
    }
}
