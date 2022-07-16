use ricq::msg::elem::{self, FlashImage, RQElem};
use ricq::msg::MessageChain;
use ricq::structs::ForwardMessage;
use ricq::Client;
use ricq_core::pb::msg::Ptt;
use tracing::{debug, warn};
use walle_core::event::{BaseEvent, Event, Message};
use walle_core::prelude::*;
use walle_core::resp::RespError;
use walle_core::segment::{self, Segments};

use crate::database::{Database, Images, SImage, Voices, WQDatabase};
use crate::error;

use super::audio::encode_to_silk;
use super::util::decode_message_id;

pub struct MsgChainBuilder<'a> {
    cli: &'a Client,
    target: i64,
    group: bool,
    message: Segments,
}

pub enum RQSendItem {
    Chain(MessageChain),
    Forward(Vec<ForwardMessage>),
    Voice(Ptt),
}

#[derive(Default)]
pub struct RQSends {
    pub chain: MessageChain,
    pub forward: Vec<ForwardMessage>,
    pub voice: Option<Ptt>,
}

impl TryFrom<RQSends> for RQSendItem {
    type Error = RespError;
    fn try_from(s: RQSends) -> Result<Self, Self::Error> {
        if !s.chain.0.is_empty() {
            Ok(RQSendItem::Chain(s.chain))
        } else if !s.forward.is_empty() {
            Ok(RQSendItem::Forward(s.forward))
        } else if let Some(voice) = s.voice {
            Ok(RQSendItem::Voice(voice))
        } else {
            Err(error::empty_message(""))
        }
    }
}

impl<'a> MsgChainBuilder<'a> {
    pub fn group_chain_builder(cli: &'a Client, target: i64, message: Segments) -> Self {
        MsgChainBuilder {
            cli,
            target,
            group: true,
            message,
        }
    }
    pub fn private_chain_builder(cli: &'a Client, target: i64, message: Segments) -> Self {
        MsgChainBuilder {
            cli,
            target,
            group: false,
            message,
        }
    }
    pub(crate) async fn build(self, wqdb: &WQDatabase) -> Result<RQSendItem, RespError> {
        let mut items = RQSends::default();
        let mut reply = None;
        for msg_seg in self.message {
            if let Some(r) =
                push_msg_seg(&mut items, msg_seg, self.target, self.group, self.cli, wqdb).await?
            {
                reply = Some(r);
            }
        }
        if let Some(r) = reply {
            items.chain.with_reply(r);
            if items.chain.0.len() == 1 {
                items.chain.push(elem::Text::new(" ".to_string()));
            }
        }
        items.try_into()
    }
}

pub(crate) fn rq_elem2msg_seg(elem: RQElem, wqdb: &WQDatabase) -> Option<MessageSegment> {
    match elem {
        RQElem::Text(text) => Some(MessageSegment {
            ty: "text".to_string(),
            data: value_map! {"text": text.content},
        }),
        RQElem::At(elem::At { target: 0, .. }) => Some(MessageSegment {
            ty: "mention_all".to_string(),
            data: value_map! {},
        }),
        RQElem::At(at) => Some(MessageSegment {
            ty: "mention".to_string(),
            data: value_map! {"user_id": at.target.to_string()},
        }),
        RQElem::Face(face) => Some(MessageSegment {
            ty: "face".to_owned(),
            data: value_map! {
                "id": face.index,
                "file": face.name
            },
        }),
        RQElem::MarketFace(face) => Some(MessageSegment {
            ty: "text".to_string(),
            data: value_map! {"text": face.name},
        }),
        RQElem::Dice(d) => Some(MessageSegment {
            ty: "dice".to_string(),
            data: value_map! {
                "value": d.value,
            },
        }),
        RQElem::FingerGuessing(f) => Some(MessageSegment {
            ty: "rps".to_string(),
            data: value_map! {
                "value": match f {
                    elem::FingerGuessing::Rock => 0i8,
                    elem::FingerGuessing::Paper => 1,
                    elem::FingerGuessing::Scissors => 2,
                }
            },
        }),
        RQElem::LightApp(l) => Some(MessageSegment {
            ty: "json".to_string(),
            data: value_map! {"data": l.content},
        }),
        RQElem::FriendImage(i) => {
            wqdb.insert_image(&i);
            Some(MessageSegment {
                ty: "image".to_string(),
                data: value_map! {
                    "file_id": i.hex_image_id(),
                    "url": i.url(),
                    "falsh": false
                },
            })
        }
        RQElem::GroupImage(i) => {
            wqdb.insert_image(&i);
            Some(MessageSegment {
                ty: "image".to_string(),
                data: value_map! {
                    "file_id": i.hex_image_id(),
                    "url": i.url(),
                    "flash": false
                },
            })
        }
        RQElem::FlashImage(fi) => match fi {
            FlashImage::FriendImage(fi) => {
                wqdb.insert_image(&fi);
                Some(MessageSegment {
                    ty: "image".to_string(),
                    data: value_map! {
                        "file_id": fi.hex_image_id(),
                        "url": fi.url(),
                        "flash": true
                    },
                })
            }
            FlashImage::GroupImage(gi) => {
                wqdb.insert_image(&gi);
                Some(MessageSegment {
                    ty: "image".to_string(),
                    data: value_map! {
                        "file_id": gi.hex_image_id(),
                        "url": gi.url(),
                        "flash": true
                    },
                })
            }
        },
        RQElem::RichMsg(rich) => Some(MessageSegment {
            ty: "xml".to_string(),
            data: value_map! {
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
        rv.push(
            segment::Reply {
                message_id: reply.reply_seq.to_string(),
                user_id: reply.sender.to_string(),
            }
            .into(),
        )
    }
    for seg in chain.into_iter().filter_map(|s| rq_elem2msg_seg(s, wqdb)) {
        rv.push(seg);
    }
    rv
}

macro_rules! maybe_flash {
    ($chain: expr, $flash: expr, $image: ident) => {
        if $flash {
            $chain.push(FlashImage::from($image));
        } else {
            $chain.push($image);
        }
    };
}

use crate::model::WQSegment;

async fn push_msg_seg(
    items: &mut RQSends,
    seg: MessageSegment,
    target: i64,
    group: bool,
    cli: &Client,
    wqdb: &WQDatabase,
) -> Result<Option<elem::Reply>, RespError> {
    match seg.try_into().map_err(|e: WalleError| match e {
        WalleError::DeclareNotMatch(_, get) => resp_error::unsupported_action(get),
        WalleError::MapMissedKey(key) => resp_error::bad_segment_data(key),
        _ => unreachable!(),
    })? {
        WQSegment::Text(text) => items.chain.push(elem::Text { content: text.text }),
        WQSegment::Mention(mention) => {
            if let Ok(user_id) = mention.user_id.parse() {
                let display = cli
                    .get_group_member_info(target, user_id)
                    .await
                    .map(|info| info.nickname)
                    .unwrap_or_else(|_| user_id.to_string());
                items.chain.push(elem::At {
                    display,
                    target: user_id,
                })
            }
        }
        WQSegment::MentionAll {} => items.chain.push(elem::At {
            display: "all".to_string(),
            target: 0,
        }),
        WQSegment::Reply(reply) => {
            let event = wqdb
                .get_message::<Event>(&reply.message_id)
                .ok_or_else(|| error::message_not_exist(&reply.message_id))?;
            let event = BaseEvent::<Message>::try_from(event).unwrap(); //todo check
            let decoded = decode_message_id(&reply.message_id)?;
            let sub_chain = {
                let mut chain = MessageChain::default();
                chain.push(elem::Text {
                    content: event.ty.alt_message,
                });
                chain
            };
            return Ok(Some(elem::Reply {
                reply_seq: *decoded.1.first().unwrap(),
                sender: event.ty.user_id.parse().unwrap(),
                time: event.time as i32,
                elements: sub_chain,
            }));
        }
        WQSegment::Face(face) => {
            if let Some(id) = face.id {
                items.chain.push(elem::Face::new(id));
            } else if let Some(face) = face.file.and_then(|name| elem::Face::new_from_name(&name)) {
                items.chain.push(face);
            } else {
                warn!("invalid face id");
                return Err(error::bad_param("face"));
            }
        }
        WQSegment::Xml(xml) => {
            items.chain.push(elem::RichMsg {
                service_id: xml.service_id,
                template1: xml.data,
            });
        }
        WQSegment::Image(image) => {
            let flash = image.flash.unwrap_or_default();
            if let Some(info) = wqdb.get_image::<Images>(
                &hex::decode(&image.file_id).map_err(|_| error::bad_param("file_id"))?,
            )? {
                if group {
                    if let Some(image) = info.try_into_group_elem(cli, target).await {
                        maybe_flash!(items.chain, flash, image);
                    }
                } else if let Some(image) = info.try_into_friend_elem(cli, target).await {
                    maybe_flash!(items.chain, flash, image);
                }
            } else if let Some(uri) = image.url {
                match uri_reader::uget(&uri).await {
                    Ok(data) => {
                        if group {
                            match cli.upload_group_image(target, data).await {
                                Ok(image) => maybe_flash!(items.chain, flash, image),
                                Err(e) => {
                                    warn!(target: crate::WALLE_Q, "群图片上传失败：{}", e);
                                    return Err(error::rq_error(e));
                                }
                            }
                        } else {
                            match cli.upload_friend_image(target, data).await {
                                Ok(image) => maybe_flash!(items.chain, flash, image),
                                Err(e) => {
                                    warn!(target: crate::WALLE_Q, "好友图片上传失败：{}", e);
                                    return Err(error::rq_error(e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("uri get failed: {}", e);
                        return Err(error::bad_param(&format!("url:{}", e)));
                    }
                }
            } else {
                warn!("image not found: {}", image.file_id);
                return Err(error::file_not_found(image.file_id));
            }
        }
        WQSegment::Voice(voice) => {
            match wqdb
                .get_voice(&hex::decode(&voice.file_id).map_err(|_| error::bad_param("file_id"))?)?
            {
                Some(Voices::Ptt(ptt)) => items.voice = Some(ptt),
                Some(Voices::Local(local)) if group => {
                    let group_audio = cli
                        .upload_group_audio(
                            target,
                            encode_to_silk(local.path().to_str().unwrap()).await?,
                            1,
                        )
                        .await
                        .map_err(|e| error::rq_error(e))?;
                    items.voice = Some(group_audio.0);
                }
                Some(Voices::Local(local)) => {
                    let friend_audio = cli
                        .upload_friend_audio(
                            target,
                            encode_to_silk(local.path().to_str().unwrap()).await?,
                            std::time::Duration::from_secs(10), //just a number tired
                        )
                        .await
                        .map_err(|e| error::rq_error(e))?;
                    items.voice = Some(friend_audio.0);
                }
                None => {
                    warn!("audio not found: {}", voice.file_id);
                    return Err(error::file_not_found(voice.file_id));
                }
            }
        }
    }
    Ok(None)
}
