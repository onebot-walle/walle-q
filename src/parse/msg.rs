use ricq::msg::elem::{self, FlashImage, RQElem};
use ricq::msg::MessageChain;
use ricq::Client;
use tracing::{debug, trace, warn};
use walle_core::{extended_map, ExtendedMapExt, Message, MessageSegment};

use crate::database::{Database, SImage, WQDatabase};

pub struct MsgChainBuilder<'a> {
    cli: &'a Client,
    target: i64,
    group: bool,
    message: Message,
}

impl<'a> MsgChainBuilder<'a> {
    pub fn group_chain_builder(cli: &'a Client, target: i64, message: Message) -> Self {
        MsgChainBuilder {
            cli,
            target,
            group: true,
            message,
        }
    }
    pub fn private_chain_builder(cli: &'a Client, target: i64, message: Message) -> Self {
        MsgChainBuilder {
            cli,
            target,
            group: false,
            message,
        }
    }
    pub(crate) async fn build(self, wqdb: &WQDatabase) -> Option<MessageChain> {
        let mut chain = MessageChain::default();
        for msg_seg in self.message {
            push_msg_seg(&mut chain, msg_seg, self.target, self.group, self.cli, wqdb).await;
        }
        if chain.0.is_empty() {
            None
        } else {
            Some(chain)
        }
    }
}

pub(crate) fn rq_elem2msg_seg(elem: RQElem, wqdb: &WQDatabase) -> Option<MessageSegment> {
    match elem {
        RQElem::Text(text) => Some(MessageSegment::text(text.content)),
        RQElem::At(elem::At { target: 0, .. }) => Some(MessageSegment::mention_all()),
        RQElem::At(at) => Some(MessageSegment::mention(at.target.to_string())),
        RQElem::Face(face) => Some(MessageSegment::Custom {
            ty: "face".to_owned(),
            data: extended_map! {
                "id": face.index,
                "file": face.name
            },
        }),
        RQElem::MarketFace(face) => Some(MessageSegment::text(face.name)),
        RQElem::Dice(d) => Some(MessageSegment::Custom {
            ty: "dice".to_owned(),
            data: extended_map! {
                "value": d.value,
            },
        }),
        RQElem::FingerGuessing(f) => Some(MessageSegment::Custom {
            ty: "rps".to_owned(),
            data: extended_map! {
                "value": match f {
                    elem::FingerGuessing::Rock => 0i8,
                    elem::FingerGuessing::Paper => 1,
                    elem::FingerGuessing::Scissors => 2,
                }
            },
        }),
        RQElem::LightApp(l) => Some(MessageSegment::Custom {
            ty: "json".to_owned(),
            data: extended_map! {"data": l.content},
        }),
        RQElem::FriendImage(i) => {
            wqdb._insert_image(&i);
            Some(MessageSegment::Image {
                extra: extended_map! {"url": i.url(), "flash": false},
                file_id: i.hex_image_id(),
            })
        }
        RQElem::GroupImage(i) => {
            wqdb._insert_image(&i);
            Some(MessageSegment::Image {
                extra: extended_map! {"url": i.url(), "flash": false},
                file_id: i.hex_image_id(),
            })
        }
        RQElem::FlashImage(fi) => match fi {
            FlashImage::FriendImage(fi) => {
                wqdb._insert_image(&fi);
                Some(MessageSegment::Image {
                    extra: extended_map! {"url": fi.url(), "flash": true},
                    file_id: fi.hex_image_id(),
                })
            }
            FlashImage::GroupImage(gi) => {
                wqdb._insert_image(&gi);
                Some(MessageSegment::Image {
                    extra: extended_map! {"url": gi.url(), "flash": true},
                    file_id: gi.hex_image_id(),
                })
            }
        },
        RQElem::Other(_) => {
            trace!("unknown Other MsgElem: {:?}", elem);
            None
        }
        elem => {
            debug!("unsupported MsgElem: {:?}", elem);
            None
        }
    }
}

pub(crate) fn msg_chain2msg_seg_vec(chain: MessageChain, wqdb: &WQDatabase) -> Vec<MessageSegment> {
    chain
        .into_iter()
        .filter_map(|s| rq_elem2msg_seg(s, wqdb))
        .collect()
}

macro_rules! maybe_flash {
    ($chain: ident, $flash: expr, $image: ident) => {
        if $flash {
            $chain.push(FlashImage::from($image));
        } else {
            $chain.push($image);
        }
    };
}

async fn push_msg_seg(
    chain: &mut MessageChain,
    seg: MessageSegment,
    target: i64,
    group: bool,
    cli: &Client,
    wqdb: &WQDatabase,
) {
    match seg {
        MessageSegment::Text { text, .. } => chain.push(elem::Text { content: text }),
        MessageSegment::Mention { user_id, .. } => {
            if let Ok(user_id) = user_id.parse() {
                let display = cli
                    .get_group_member_info(target, user_id)
                    .await
                    .map(|info| info.nickname)
                    .unwrap_or(user_id.to_string());
                chain.push(elem::At {
                    display,
                    target: user_id,
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
        MessageSegment::Image { file_id, mut extra } => {
            let flash = extra.try_remove("flash").unwrap_or(false);
            if let Some(info) = hex::decode(&file_id)
                .ok()
                .and_then(|id| wqdb.get_image(&id))
            {
                if group {
                    if let Some(image) = info.try_into_group_elem(cli, target).await {
                        maybe_flash!(chain, flash, image);
                    }
                } else if let Some(image) = info.try_into_friend_elem(cli, target).await {
                    maybe_flash!(chain, flash, image);
                }
            } else if let Some(uri) = extra.remove("url").and_then(|b64| b64.downcast_str().ok()) {
                match uri_reader::uget(&uri).await {
                    Ok(data) => {
                        if group {
                            match cli.upload_group_image(target, data).await {
                                Ok(image) => maybe_flash!(chain, flash, image),
                                Err(e) => warn!(target: crate::WALLE_Q, "群图片上传失败：{}", e),
                            }
                        } else {
                            match cli.upload_friend_image(target, data).await {
                                Ok(image) => maybe_flash!(chain, flash, image),
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
            Self::Text { .. }
            | Self::Mention { .. }
            | Self::MentionAll { .. }
            | Self::Image { .. } => true,
            Self::Custom { ty, .. } if ty == "face" => true,
            _ => false,
        }
    }
}
