use ricq::msg::elem::{self, FlashImage, RQElem};
use ricq::msg::MessageChain;
use ricq::structs::ForwardMessage;
use ricq::Client;
use ricq_core::pb::msg::Ptt;
use tracing::{debug, warn};
use walle_core::resp::RespError;
use walle_core::{
    extended_map, ExtendedMapExt, ExtendedValue, Message, MessageEvent, MessageSegment,
};

use crate::database::{Database, Images, SImage, Voices, WQDatabase};
use crate::error;
use crate::extra::segment::NodeEnum;
use crate::extra::ToMessageEvent;

use super::audio::encode_to_silk;

pub struct MsgChainBuilder<'a> {
    cli: &'a Client,
    target: i64,
    group: bool,
    message: Message,
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
            Err(error::empty_message())
        }
    }
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
            wqdb.insert_image(&i);
            Some(MessageSegment::Image {
                extra: extended_map! {"url": i.url(), "flash": false},
                file_id: i.hex_image_id(),
            })
        }
        RQElem::GroupImage(i) => {
            wqdb.insert_image(&i);
            Some(MessageSegment::Image {
                extra: extended_map! {"url": i.url(), "flash": false},
                file_id: i.hex_image_id(),
            })
        }
        RQElem::FlashImage(fi) => match fi {
            FlashImage::FriendImage(fi) => {
                wqdb.insert_image(&fi);
                Some(MessageSegment::Image {
                    extra: extended_map! {"url": fi.url(), "flash": true},
                    file_id: fi.hex_image_id(),
                })
            }
            FlashImage::GroupImage(gi) => {
                wqdb.insert_image(&gi);
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
    ($chain: expr, $flash: expr, $image: ident) => {
        if $flash {
            $chain.push(FlashImage::from($image));
        } else {
            $chain.push($image);
        }
    };
}

async fn push_msg_seg(
    items: &mut RQSends,
    seg: MessageSegment,
    target: i64,
    group: bool,
    cli: &Client,
    wqdb: &WQDatabase,
) -> Result<Option<elem::Reply>, RespError> {
    match seg {
        MessageSegment::Text { text, .. } => items.chain.push(elem::Text { content: text }),
        MessageSegment::Mention { user_id, .. } => {
            if let Ok(user_id) = user_id.parse() {
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
        MessageSegment::MentionAll { .. } => items.chain.push(elem::At {
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
                .to_message_event()
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
                time: event.time as i32,
                elements: sub_chain,
            }));
        }
        MessageSegment::Custom { ty, mut data } => match ty.as_str() {
            "face" => {
                if let Some(id) = data.remove("id").and_then(|v| v.downcast_int().ok()) {
                    items.chain.push(elem::Face::new(id as i32));
                } else if let Some(face) = data
                    .remove("file")
                    .and_then(|v| v.downcast_str().ok())
                    .and_then(|name| elem::Face::new_from_name(&name))
                {
                    items.chain.push(face);
                } else {
                    warn!("invalid face id");
                    return Err(error::bad_param("face"));
                }
            }
            "xml" => {
                let service_id =
                    data.try_remove::<i64>("service_id")
                        .map_err(|_| error::bad_param("service_id"))? as i32;
                let template1: String = data
                    .try_remove("data")
                    .map_err(|_| error::bad_param("data"))?;
                items.chain.push(elem::RichMsg {
                    service_id,
                    template1,
                });
            }
            "forward" => {
                let nodes: Vec<NodeEnum> = serde_json::from_str(
                    &serde_json::to_string(
                        &data
                            .try_remove::<Vec<ExtendedValue>>("nodes")
                            .map_err(|_| error::bad_param("nodes"))?,
                    )
                    .unwrap(),
                )
                .map_err(|_| error::bad_param("nodes"))?;
                for node in nodes {
                    items.forward.push(match node {
                        NodeEnum::Node(n) => n.to_forward_message(target, group, cli, wqdb).await?,
                    })
                }
            }
            _ => {
                warn!("unsupported custom type: {}", ty);
                return Err(walle_core::resp::error_builder::unsupported_segment());
            }
        },
        MessageSegment::Image { file_id, mut extra } => {
            let flash = extra.try_remove("flash").unwrap_or(false);
            if let Some(info) = wqdb.get_image::<Images>(
                &hex::decode(&file_id).map_err(|_| error::bad_param("file_id"))?,
            )? {
                if group {
                    if let Some(image) = info.try_into_group_elem(cli, target).await {
                        maybe_flash!(items.chain, flash, image);
                    }
                } else if let Some(image) = info.try_into_friend_elem(cli, target).await {
                    maybe_flash!(items.chain, flash, image);
                }
            } else if let Some(uri) = extra.remove("url").and_then(|b64| b64.downcast_str().ok()) {
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
                warn!("image not found: {}", file_id);
                return Err(error::file_not_found());
            }
        }
        MessageSegment::Voice { file_id, .. } => {
            match wqdb
                .get_voice(&hex::decode(&file_id).map_err(|_| error::bad_param("file_id"))?)?
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
                    warn!("audio not found: {}", file_id);
                    return Err(error::file_not_found());
                }
            }
        }
        seg => {
            warn!("unsupported MessageSegment: {:?}", seg);
            return Err(walle_core::resp::error_builder::unsupported_segment());
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
