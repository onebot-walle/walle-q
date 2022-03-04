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
            extend: [("url".to_string(), i.url().into())].into(),
            file_id: i.image_id,
        }),
        RQElem::GroupImage(i) => Some(MessageSegment::Image {
            extend: [("url".to_string(), i.url().into())].into(),
            file_id: i.image_id,
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

trait PushMsgSeg {
    fn push_msg_seg(&mut self, seg: MessageSegment);
}

impl PushMsgSeg for MessageChain {
    fn push_msg_seg(&mut self, seq: MessageSegment) {
        match seq {
            MessageSegment::Text { text, .. } => self.push(elem::Text { content: text }),
            MessageSegment::Mention { user_id, .. } => {
                if let Ok(target) = user_id.parse() {
                    self.push(elem::At {
                        display: user_id.to_string(),
                        target,
                    })
                }
            }
            MessageSegment::MentionAll { .. } => self.push(elem::At {
                display: "all".to_string(),
                target: 0,
            }),
            MessageSegment::Custom { ty, mut data } => match ty.as_str() {
                "face" => {
                    if let Some(id) = data.remove("id").and_then(|v| v.downcast_int().ok()) {
                        self.push(elem::Face::new(id as i32));
                    } else {
                        warn!("invalid face id");
                    }
                }
                _ => warn!("unsupported custom type: {}", ty),
            },
            seg => {
                warn!("unsupported MessageSegment: {:?}", seg);
            }
        }
    }
}

pub fn msg_seg_vec2msg_chain(v: Vec<MessageSegment>) -> MessageChain {
    let mut chain = MessageChain::default();
    for msg_seg in v {
        chain.push_msg_seg(msg_seg);
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
            Self::Custom { ty, .. } if ty == "face" => true,
            _ => false,
        }
    }
}
