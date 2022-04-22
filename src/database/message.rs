use rs_qq::structs::{FriendMessage, GroupMessage, MessageReceipt};
use serde::{Deserialize, Serialize};
use walle_core::Message;

pub trait MessageId {
    fn seq(&self) -> i32;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SMessage {
    Group(SGroupMessage),
    Private(SPrivateMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SGroupMessage {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub group_code: i64,
    pub from_uin: i64,
    pub time: i32,
    pub message: Message,
}

impl MessageId for SGroupMessage {
    fn seq(&self) -> i32 {
        self.seqs[0]
    }
}

impl SGroupMessage {
    pub fn new(m: GroupMessage, message: Message) -> Self {
        Self {
            seqs: m.seqs,
            rands: m.rands,
            group_code: m.group_code,
            from_uin: m.from_uin,
            time: m.time,
            message,
        }
    }

    pub fn receipt(
        receipt: MessageReceipt,
        group_code: i64,
        from_uin: i64,
        message: Message,
    ) -> Self {
        Self {
            seqs: receipt.seqs,
            rands: receipt.rands,
            group_code,
            from_uin,
            time: receipt.time as i32,
            message,
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
    pub message: Message,
}

impl MessageId for SPrivateMessage {
    fn seq(&self) -> i32 {
        self.seqs[0]
    }
}

impl SPrivateMessage {
    pub fn new(m: FriendMessage, message: Message) -> Self {
        Self {
            seqs: m.seqs,
            rands: m.rands,
            target: m.target,
            from_uin: m.from_uin,
            from_nick: m.from_nick,
            time: m.time,
            message,
        }
    }

    pub fn receipt(
        receipt: MessageReceipt,
        target: i64,
        from_uin: i64,
        from_nick: String,
        message: Message,
    ) -> Self {
        Self {
            seqs: receipt.seqs,
            rands: receipt.rands,
            target,
            from_uin,
            from_nick,
            time: receipt.time as i32,
            message,
        }
    }
}
