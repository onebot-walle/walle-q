use ricq::msg::elem::{self, FlashImage, RQElem};
use ricq::msg::MessageChain;
use ricq::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use walle_core::resp::RespError;
use walle_core::{extended_map, ExtendedMapExt, Message, MessageEvent, MessageSegment};

use crate::database::{Database, SImage, WQDatabase};
use crate::error;

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
    pub(crate) async fn build(self, wqdb: &WQDatabase) -> Result<Option<MessageChain>, RespError> {
        let mut chain = MessageChain::default();
        let mut reply = None;
        for msg_seg in self.message {
            if let Some(r) =
                push_msg_seg(&mut chain, msg_seg, self.target, self.group, self.cli, wqdb).await?
            {
                reply = Some(r);
            }
        }
        if let Some(r) = reply {
            chain.with_reply(r);
            if chain.0.len() == 1 {
                chain.push(elem::Text::new(" ".to_string()));
            }
        }
        if chain.0.is_empty() {
            Ok(None)
        } else {
            Ok(Some(chain))
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
        RQElem::RichMsg(rich) => Some(MessageSegment::Custom {
            ty: "xml".to_string(),
            data: extended_map! {
                "service_id": rich.service_id,
                "data": rich.template1
            },
        }),
        RQElem::Other(_) => {
            tracing::trace!(target: crate::WALLE_Q, "unknown Other MsgElem: {:?}", elem);
            None
        }
        elem => {
            debug!(target: crate::WALLE_Q, "unsupported MsgElem: {:?}", elem);
            None
        }
    }
}

pub(crate) fn msg_chain2msg_seg_vec(chain: MessageChain, wqdb: &WQDatabase) -> Vec<MessageSegment> {
    let mut rv = vec![];
    if let Some(reply) = chain.reply() {
        rv.push(MessageSegment::Reply {
            message_id: reply.reply_seq.to_string(),
            user_id: reply.sender.to_string(),
            extra: extended_map! {},
        })
    }
    for seg in chain.into_iter().filter_map(|s| rq_elem2msg_seg(s, wqdb)) {
        rv.push(seg);
    }
    rv
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
) -> Result<Option<elem::Reply>, RespError> {
    match seg {
        MessageSegment::Text { text, .. } => chain.push(elem::Text { content: text }),
        MessageSegment::Mention { user_id, .. } => {
            if let Ok(user_id) = user_id.parse() {
                let display = cli
                    .get_group_member_info(target, user_id)
                    .await
                    .map(|info| info.nickname)
                    .unwrap_or_else(|_| user_id.to_string());
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
        MessageSegment::Reply { message_id, .. } => {
            let reply_seq: i32 = message_id
                .parse()
                .map_err(|_| error::bad_param("message_id"))?;
            let event: MessageEvent = wqdb
                .get_message(reply_seq)
                .ok_or_else(error::message_not_exist)?
                .event()
                .try_into()
                .unwrap();
            let sub_chain = {
                let mut chain = MessageChain::default();
                chain.push(elem::Text {
                    content: event.alt_message().to_string(),
                });
                chain
            };
            return Ok(Some(elem::Reply {
                reply_seq,
                sender: event.user_id().parse().unwrap(),
                group_id: 0,
                time: event.time as i32,
                elements: sub_chain,
            }));
        }
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
            "xml" => {
                let service_id =
                    data.try_remove::<i64>("service_id")
                        .map_err(|_| error::bad_param("service_id"))? as i32;
                let template1: String = data
                    .try_remove("data")
                    .map_err(|_| error::bad_param("data"))?;
                chain.push(elem::RichMsg {
                    service_id,
                    template1,
                });
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
    Ok(None)
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

#[derive(Serialize, Deserialize, Debug)]
pub enum MaybeNode {
    Node(),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Forward {}
