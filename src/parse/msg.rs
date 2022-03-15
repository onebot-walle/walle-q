use crate::database::{Database, SImage};
use crate::parse::WQResult;
use rs_qq::msg::elem::{self, RQElem};
use rs_qq::msg::MessageChain;
use rs_qq::structs::ImageInfo;
use rs_qq::Client;
use tracing::{debug, warn};
use walle_core::{Message, MessageSegment};

pub struct MsgChainBuilder<'a> {
    cli: &'a Client,
    target: i64,
    group: bool,
    message: Message,
}

impl<'a> MsgChainBuilder<'a> {
    pub fn group_chain_builder(cli: &'a rs_qq::Client, target: i64, message: Message) -> Self {
        MsgChainBuilder {
            cli,
            target,
            group: true,
            message,
        }
    }
    pub fn private_chain_builder(cli: &'a rs_qq::Client, target: i64, message: Message) -> Self {
        MsgChainBuilder {
            cli,
            target,
            group: false,
            message,
        }
    }
    pub async fn build(self) -> WQResult<MessageChain> {
        let mut chain = MessageChain::default();
        for msg_seg in self.message {
            push_msg_seg(&mut chain, msg_seg, self.target, self.group, self.cli).await?;
        }
        Ok(chain)
    }
}

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
        RQElem::GroupImage(i) => {
            let info = SImage::from(ImageInfo::from(i.clone()));
            crate::SLED_DB.insert_image(&info);
            Some(MessageSegment::Image {
                extend: [("url".to_string(), i.url().into())].into(),
                file_id: i.image_id,
            })
        }
        elem => {
            debug!("unsupported MsgElem: {:?}", elem);
            None
        }
    }
}

pub fn msg_chain2msg_seg_vec(chain: MessageChain) -> Vec<MessageSegment> {
    chain.into_iter().filter_map(rq_elem2msg_seg).collect()
}

async fn push_msg_seg(
    chain: &mut MessageChain,
    seq: MessageSegment,
    target: i64,
    group: bool,
    cli: &Client,
) -> WQResult<()> {
    match seq {
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
        MessageSegment::Custom { ty, mut data } => match ty.as_str() {
            "face" => {
                if let Some(id) = data.remove("id").and_then(|v| v.downcast_int().ok()) {
                    chain.push(elem::Face::new(id as i32));
                } else {
                    warn!("invalid face id");
                }
            }
            _ => warn!("unsupported custom type: {}", ty),
        },
        MessageSegment::Image { file_id, extend: _ } => {
            if let Some(info) = crate::SLED_DB.get_image(&file_id) {
                if group {
                    chain.push(info.try_into_group_elem(cli, target).await?);
                } else {
                    chain.push(info.try_into_private_elem(cli, target).await?);
                }
            } else {
                warn!("image not found: {}", file_id);
            }
        }
        seg => {
            warn!("unsupported MessageSegment: {:?}", seg);
        }
    }
    Ok(())
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
