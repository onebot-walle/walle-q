use crate::parse::SendAble;
use rq_engine::{
    msg::MessageChain,
    structs::{GroupMessage, PrivateMessage},
};
use serde::{Deserialize, Serialize};
use walle_core::{BaseEvent, Event, Message, MessageContent};

pub(crate) mod sled;

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database: DatabaseInit + Sized {
    fn get_message_event(&self, key: &str) -> Option<BaseEvent<MessageContent>>;
    fn insert_message_event(&self, value: &Event);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SGroupMessage {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub group_code: i64,
    pub from_uin: i64,
    pub time: i32,
    pub elements: Message,
}

impl From<GroupMessage> for SGroupMessage {
    fn from(m: GroupMessage) -> Self {
        Self {
            seqs: m.seqs,
            rands: m.rands,
            group_code: m.group_code,
            from_uin: m.from_uin,
            time: m.time,
            elements: parse_only_sendable_seq(m.elements),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SPrivateMessage {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub target: i64,
    pub time: i32,
    pub from_uin: i64,
    pub from_nick: String,
    pub elements: Message,
}

impl From<PrivateMessage> for SPrivateMessage {
    fn from(m: PrivateMessage) -> Self {
        Self {
            seqs: m.seqs,
            rands: m.rands,
            target: m.target,
            from_uin: m.from_uin,
            from_nick: m.from_nick,
            time: m.time,
            elements: parse_only_sendable_seq(m.elements),
        }
    }
}

fn parse_only_sendable_seq(chain: MessageChain) -> Message {
    chain
        .into_iter()
        .filter_map(|e| crate::parse::rq_elem2msg_seg(e))
        .filter(SendAble::check)
        .collect()
}
