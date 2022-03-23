use crate::database::{Database, SImage};
use rs_qq::msg::elem::{self, RQElem};
use rs_qq::msg::MessageChain;
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
    pub async fn build(self) -> Option<MessageChain> {
        let mut chain = MessageChain::default();
        for msg_seg in self.message {
            push_msg_seg(&mut chain, msg_seg, self.target, self.group, self.cli).await;
        }
        if chain.0.is_empty() {
            None
        } else {
            Some(chain)
        }
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
                        elem::FingerGuessing::Rock => 0i8,
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
        RQElem::FriendImage(i) => {
            crate::WQDB._insert_image(&i);
            Some(MessageSegment::Image {
                extend: [("url".to_string(), i.url().into())].into(),
                file_id: i.hex_image_id(),
            })
        }
        RQElem::GroupImage(i) => {
            crate::WQDB._insert_image(&i);
            Some(MessageSegment::Image {
                extend: [("url".to_string(), i.url().into())].into(),
                file_id: i.hex_image_id(),
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
    seg: MessageSegment,
    target: i64,
    group: bool,
    cli: &Client,
) {
    match seg {
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
                } else if let Some(face) = data
                    .remove("file")
                    .and_then(|v| v.downcast_str().ok())
                    .and_then(|name| elem::Face::new_from_name(&name))
                {
                    chain.push(face);
                } else {
                    warn!("invalid face id");
                }
            }
            _ => warn!("unsupported custom type: {}", ty),
        },
        MessageSegment::Image {
            file_id,
            mut extend,
        } => {
            if let Some(info) = hex::decode(&file_id)
                .ok()
                .and_then(|id| crate::WQDB.get_image(&id))
            {
                if group {
                    if let Some(image) = info.try_into_group_elem(cli, target).await {
                        chain.push(image);
                    }
                } else if let Some(image) = info.try_into_friend_elem(cli, target).await {
                    chain.push(image);
                }
            } else if let Some(b64) = extend.remove("url").and_then(|b64| b64.downcast_str().ok()) {
                match uri_reader::uget(&b64).await {
                    Ok(data) => {
                        if group {
                            match cli.upload_group_image(target, data).await {
                                Ok(image) => chain.push(image),
                                Err(e) => warn!(target: crate::WALLE_Q, "群图片上传失败：{}", e),
                            }
                        } else {
                            match cli.upload_friend_image(target, data).await {
                                Ok(image) => chain.push(image),
                                Err(e) => warn!(target: crate::WALLE_Q, "好友图片上传失败：{}", e),
                            }
                        }
                    }
                    Err(e) => {
                        warn!("uri get failed: {}", e);
                    }
                }
            } else {
                warn!("image not found: {}", file_id);
            }
        }
        seg => {
            warn!("unsupported MessageSegment: {:?}", seg);
        }
    }
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
