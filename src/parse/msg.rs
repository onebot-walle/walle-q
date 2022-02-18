use rs_qq::msg::elem::{self, RQElem};
use rs_qq::msg::MessageChain;
use tracing::{debug, warn};
use walle_core::MessageSegment;

pub fn rq_elem2msg_seg(elem: RQElem) -> Option<MessageSegment> {
    match elem {
        RQElem::Text(text) => Some(MessageSegment::text(text.content)),
        RQElem::At(elem::At { target: 0, .. }) => Some(MessageSegment::mention_all()),
        RQElem::At(at) => Some(MessageSegment::mention(at.target.to_string())),
        RQElem::Face(face) => Some(MessageSegment::Custom {
            ty: "face".to_owned(),
            data: [
                ("id".to_string(), (face.index as i64).into()),
                ("file".to_string(), face.name.into()),
            ]
            .into(),
        }),
        RQElem::MarketFace(face) => Some(MessageSegment::text(face.name)),
        RQElem::Dice(d) => Some(MessageSegment::Custom {
            ty: "dice".to_owned(),
            data: [("value".to_string(), (d.value as i64).into())].into(),
        }),
        RQElem::FingerGuessing(f) => Some(MessageSegment::Custom {
            ty: "rps".to_owned(),
            data: [(
                "value".to_string(),
                {
                    match f {
                        elem::FingerGuessing::Rock => 0,
                        elem::FingerGuessing::Scissors => 1,
                        elem::FingerGuessing::Paper => 2,
                    }
                }
                .into(),
            )]
            .into(),
        }),
        RQElem::LightApp(l) => Some(MessageSegment::Custom {
            ty: "json".to_owned(),
            data: [("data".to_string(), l.content.into())].into(),
        }),
        RQElem::FriendImage(i) => Some(MessageSegment::Image {
            file_id: i.image_id,
            extend: [("url".to_string(), i.url.into())].into(),
        }),
        RQElem::GroupImage(i) => Some(MessageSegment::Image {
            file_id: i.image_id,
            extend: [("url".to_string(), i.url.into())].into(),
        }),
        elem => {
            debug!("unsupported MsgElem: {:?}", elem);
            None
        }
    }
}

pub fn msg_chain2msg_seg_vec(chain: MessageChain) -> Vec<MessageSegment> {
    chain.into_iter().filter_map(rq_elem2msg_seg).collect()
}

pub fn msg_seg_vec2msg_chain(v: Vec<MessageSegment>) -> MessageChain {
    let mut chain = MessageChain::default();
    for msg_seg in v {
        match msg_seg {
            MessageSegment::Text { text, .. } => chain.push(elem::Text { content: text }),
            MessageSegment::Mention { user_id, .. } => {
                if let Ok(target) = user_id.parse() {
                    chain.push(elem::At {
                        display: user_id.to_string(),
                        target,
                    })
                }
            }
            MessageSegment::MentionAll { .. } => chain.push(elem::At {
                display: "all".to_string(),
                target: 0,
            }),
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

pub trait SendAble {
    fn check(&self) -> bool;
}

impl SendAble for MessageSegment {
    fn check(&self) -> bool {
        match self {
            Self::Text { .. } | Self::Mention { .. } | Self::MentionAll { .. } => true,
            _ => false,
        }
    }
}
