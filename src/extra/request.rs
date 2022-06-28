use serde::{Deserialize, Serialize};
use walle_core::event::EventType;
use walle_core::util::ColoredAlt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum WQRequestContent {
    NewFriend {
        sub_type: String,
        request_id: i64,
        user_id: String,
        user_name: String,
        message: String,
    },
    JoinGroup {
        sub_type: String,
        request_id: i64,
        user_id: String,
        user_name: String,
        group_id: String,
        group_name: String,
        message: String,
        suspicious: bool,
        invitor_id: Option<String>,
        invitor_name: Option<String>,
    },
    GroupInvited {
        sub_type: String,
        request_id: i64,
        group_id: String,
        group_name: String,
        invitor_id: String,
        invitor_name: String,
    },
}

impl EventType for WQRequestContent {
    fn event_type(&self) -> &str {
        "request"
    }
    fn detail_type(&self) -> &str {
        match self {
            WQRequestContent::NewFriend { .. } => "new_friend",
            WQRequestContent::JoinGroup { .. } => "join_group_request",
            WQRequestContent::GroupInvited { .. } => "group_invited",
        }
    }
    fn sub_type(&self) -> &str {
        match self {
            WQRequestContent::NewFriend { sub_type, .. } => sub_type,
            WQRequestContent::JoinGroup { sub_type, .. } => sub_type,
            WQRequestContent::GroupInvited { sub_type, .. } => sub_type,
        }
    }
}

impl ColoredAlt for WQRequestContent {
    fn colored_alt(&self) -> Option<String> {
        use colored::*;
        let head = format!("[{}]", self.detail_type().bright_cyan());
        let body = format!("{:?}", self);
        Some(format!("{} {}", head, body))
    }
}

// impl From<>
