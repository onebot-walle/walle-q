use serde::{Deserialize, Serialize};
use walle_core::{ColoredAlt, NoticeContent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum WQExtraNoticeContent {
    FriendPock {
        sub_type: String,
        user_id: String,
        receiver_id: String,
    },
    GroupNameUpdate {
        sub_type: String,
        group_id: String,
        group_name: String,
        operator_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum WQNoticeContent {
    Standard(NoticeContent),
    Extra(WQExtraNoticeContent),
}

impl From<NoticeContent> for WQNoticeContent {
    fn from(content: NoticeContent) -> Self {
        WQNoticeContent::Standard(content)
    }
}

impl From<WQExtraNoticeContent> for WQNoticeContent {
    fn from(content: WQExtraNoticeContent) -> Self {
        WQNoticeContent::Extra(content)
    }
}

impl From<NoticeContent> for super::WQEventContent {
    fn from(content: NoticeContent) -> Self {
        super::WQEventContent::Notice(WQNoticeContent::from(content))
    }
}

impl From<WQExtraNoticeContent> for super::WQEventContent {
    fn from(content: WQExtraNoticeContent) -> Self {
        super::WQEventContent::Notice(WQNoticeContent::from(content))
    }
}

impl ColoredAlt for WQNoticeContent {
    fn colored_alt(&self) -> Option<String> {
        match self {
            WQNoticeContent::Standard(content) => content.colored_alt(),
            WQNoticeContent::Extra(content) => content.colored_alt(),
        }
    }
}

impl ColoredAlt for WQExtraNoticeContent {
    fn colored_alt(&self) -> Option<String> {
        Some(format!("{:?}", self)) //todo
    }
}
