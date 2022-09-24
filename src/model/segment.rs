use walle_core::{
    prelude::{TryFromMsgSegment, TryFromValue},
    segment::{self, MsgSegment},
};

#[derive(Debug, Clone, TryFromValue)]
pub struct Face {
    pub id: Option<i32>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct Xml {
    pub service_id: i32,
    pub data: String,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct Image {
    pub file_id: String,
    pub url: Option<String>,
    pub flash: Option<bool>,
}

#[derive(Debug, Clone, TryFromValue)]
pub struct Node {
    pub user_id: String,
    pub time: f64,
    pub user_name: String,
    pub message: Vec<MsgSegment>,
}

#[derive(Debug, Clone, TryFromMsgSegment)]
pub enum WQSegment {
    Text(segment::Text),
    MentionAll {},
    Mention(segment::Mention),
    Reply(segment::Reply),
    Face(Face),
    Image(Image),
    Xml(Xml),
    Voice(segment::Voice),
    Node(Node),
}
