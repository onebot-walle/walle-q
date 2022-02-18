use rs_qq::msg::MessageChain;
use rs_qq::structs::GroupMessage;
use serde::{Deserialize, Serialize};
use walle_core::{BaseEvent, Event, MessageContent};

pub(crate) mod sled;

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database: DatabaseInit + Sized {
    fn get_message_event(&self, key: &str) -> Option<BaseEvent<MessageContent>>;
    fn insert_message_event(&self, value: &Event);
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "GroupMessage")]
pub struct GroupMessageDef {
    pub seqs: Vec<i32>,
    pub rands: Vec<i32>,
    pub group_code: i64,
    pub from_uin: i64,
    pub time: i32,
    #[serde(with = "MessageChainDef")]
    pub elements: MessageChain,
}

pub struct MessageChainDef(pub Vec<rq_engine::pb::msg::elem::Elem>);

impl MessageChainDef {
    pub fn serialize<S>(chain: &MessageChain, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use prost::Message;
        let elems = chain
            .0
            .iter()
            .map(|e| {
                rq_engine::pb::msg::Elem {
                    elem: Some(e.clone()),
                }
                .encode_to_vec()
            })
            .collect();
        serializer.serialize_bytes()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<MessageChain, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = deserializer.deserialize_bytes()?;
        let elems = bytes
            .into_iter()
            .map(|e| rq_engine::pb::msg::elem::Elem::decode_from_vec(&e).unwrap())
            .collect();
        Ok(MessageChain(elems))
    }
}
