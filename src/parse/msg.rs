use ricq::msg::elem::{self, FlashImage, RQElem};
use ricq::msg::{MessageChain, MessageElem};
use ricq::structs::{ForwardMessage, ForwardNode, MessageNode};
use ricq::Client;
use ricq_core::pb::msg::Ptt;
use tracing::{debug, warn};
use walle_core::event::{BaseEvent, Message};
use walle_core::prelude::*;
use walle_core::resp::RespError;
use walle_core::segment::{self, Segments};

use crate::database::{Database, Images, SImage, Voices, WQDatabase};
use crate::error;
use crate::handler::Handler;
use crate::model::WQSegment;

use super::audio::encode_to_silk;

pub struct MsgChainBuilder<'a> {
    pub cli: &'a Client,
    pub db: &'a WQDatabase,
    pub target: i64,
    pub group: bool,
    data_path: &'a str,
    results: RQSends,
    reply: bool,
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
    pub fn group_chain_builder(cli: &'a Client, handler: &'a Handler, target: i64) -> Self {
        MsgChainBuilder {
            cli,
            db: &handler.database,
            data_path: &handler.data_path,
            target,
            group: true,
            results: RQSends::default(),
            reply: false,
        }
    }
    pub fn private_chain_builder(cli: &'a Client, handler: &'a Handler, target: i64) -> Self {
        MsgChainBuilder {
            cli,
            db: &handler.database,
            data_path: &handler.data_path,
            target,
            group: false,
            results: RQSends::default(),
            reply: false,
        }
    }
    pub(crate) async fn build(mut self, message: Segments) -> Result<RQSendItem, RespError> {
        for seg in message {
            self.push_seg(seg).await?;
        }
        if self.reply {
            if self.results.chain.0.len() == 1 {
                self.results.chain.push(elem::Text::new(" ".to_string()));
            }
        }
        self.results.try_into()
    }
    #[async_recursion::async_recursion]
    pub(crate) async fn push_seg(&mut self, seg: MsgSegment) -> Result<(), RespError> {
        match seg.try_into().map_err(|e: WalleError| match e {
            WalleError::DeclareNotMatch(_, get) => resp_error::unsupported_action(get),
            WalleError::MapMissedKey(key) => resp_error::bad_segment_data(key),
            _ => unreachable!(),
        })? {
            WQSegment::Text(text) => Ok(self.results.chain.push(elem::Text { content: text.text })),
            WQSegment::Mention(mention) => {
                if let Ok(user_id) = mention.user_id.parse() {
                    let display = format!(
                        "@{}",
                        if self.group {
                            self.cli
                                .get_group_member_info(self.target, user_id)
                                .await
                                .map(|info| info.nickname)
                                .unwrap_or_else(|_| user_id.to_string())
                        } else {
                            user_id.to_string()
                        }
                    );
                    Ok(self.results.chain.push(elem::At {
                        display,
                        target: user_id,
                    }))
                } else {
                    Err(error::bad_param("user_id should be int"))
                }
            }
            WQSegment::MentionAll {} => Ok(self.results.chain.push(elem::At {
                display: "all".to_string(),
                target: 0,
            })),
            WQSegment::Reply(reply) => {
                let db_event = self
                    .db
                    .get_message(&reply.message_id)
                    .ok_or_else(|| error::message_not_exist(&reply.message_id))?;
                let event = BaseEvent::<Message>::try_from(db_event.event).unwrap(); //todo check
                let sub_chain = {
                    let mut chain = MessageChain::default();
                    chain.push(elem::Text {
                        content: event.ty.alt_message,
                    });
                    chain
                };
                self.reply = true;
                Ok(self.results.chain.with_reply(elem::Reply {
                    reply_seq: *db_event.seqs.first().unwrap(),
                    sender: event.ty.user_id.parse().unwrap(),
                    time: event.time as i32,
                    elements: sub_chain,
                }))
            }
            WQSegment::Face(face) => {
                if let Some(id) = face.id {
                    Ok(self.results.chain.push(elem::Face::new(id)))
                } else if let Some(face) =
                    face.file.and_then(|name| elem::Face::new_from_name(&name))
                {
                    Ok(self.results.chain.push(face))
                } else {
                    warn!("invalid face id");
                    return Err(error::bad_param("face"));
                }
            }
            WQSegment::Xml(xml) => Ok(self.results.chain.push(elem::RichMsg {
                service_id: xml.service_id,
                template1: xml.data,
            })),
            WQSegment::Image(image) => {
                let flash = image.flash.unwrap_or_default();
                if let Some(image) = self.db.get_image::<Images>(
                    &hex::decode(&image.file_id).map_err(|_| error::bad_param("file_id"))?,
                )? {
                    self.push_image(image, flash).await
                } else if let Some(uri) = image.url {
                    match uri_reader::uget(&uri).await {
                        Ok(data) => self.push_image_data(data, flash).await,
                        Err(e) => {
                            warn!("uri get failed: {}", e);
                            Err(error::bad_param(&format!("url:{}", e)))
                        }
                    }
                } else if let Some(OneBotBytes(data)) = image.bytes {
                    self.push_image_data(data, flash).await
                } else {
                    warn!("image not found: {}", image.file_id);
                    Err(error::file_not_found(image.file_id))
                }
            }
            WQSegment::Voice(voice) => {
                match self.db.get_voice(
                    &hex::decode(&voice.file_id).map_err(|_| error::bad_param("file_id"))?,
                )? {
                    Some(Voices::Ptt(ptt)) => Ok(self.results.voice = Some(ptt)),
                    Some(Voices::Local(local)) if self.group => {
                        let group_audio = self
                            .cli
                            .upload_group_audio(
                                self.target,
                                encode_to_silk(local.path(self.data_path).to_str().unwrap())
                                    .await?,
                                1,
                            )
                            .await
                            .map_err(|e| error::rq_error(e))?;
                        Ok(self.results.voice = Some(group_audio.0))
                    }
                    Some(Voices::Local(local)) => {
                        let friend_audio = self
                            .cli
                            .upload_friend_audio(
                                self.target,
                                encode_to_silk(local.path(self.data_path).to_str().unwrap())
                                    .await?,
                                std::time::Duration::from_secs(10), //just a number tired
                            )
                            .await
                            .map_err(|e| error::rq_error(e))?;
                        Ok(self.results.voice = Some(friend_audio.0))
                    }
                    None => {
                        warn!("audio not found: {}", voice.file_id);
                        return Err(error::file_not_found(voice.file_id));
                    }
                }
            }
            WQSegment::Node(node) => {
                let sub_builder = MsgChainBuilder {
                    cli: self.cli,
                    target: self.target,
                    data_path: self.data_path,
                    group: self.group,
                    db: self.db,
                    results: RQSends::default(),
                    reply: false,
                };
                let sender_id = node
                    .user_id
                    .parse()
                    .map_err(|_| resp_error::bad_segment_data("user_id"))?;
                let time = (node.time / 1000.0) as i32;
                match sub_builder.build(node.message).await? {
                    RQSendItem::Chain(chain) => {
                        Ok(self
                            .results
                            .forward
                            .push(ForwardMessage::Message(MessageNode {
                                sender_id,
                                sender_name: node.user_name,
                                time,
                                elements: chain,
                            })))
                    }
                    RQSendItem::Forward(forwards) => {
                        Ok(self
                            .results
                            .forward
                            .push(ForwardMessage::Forward(ForwardNode {
                                sender_id,
                                sender_name: node.user_name,
                                time,
                                nodes: forwards,
                            })))
                    }
                    RQSendItem::Voice(_) => {
                        Ok(self
                            .results
                            .forward
                            .push(ForwardMessage::Message(MessageNode {
                                sender_id,
                                sender_name: node.user_name,
                                time,
                                elements: {
                                    let mut chain = MessageChain::default();
                                    chain.push(ricq::msg::elem::Text::new("[语音]".to_string()));
                                    chain
                                },
                            })))
                    }
                }
            }
        }
    }
    pub(crate) fn push_flash<T: Into<FlashImage> + Into<Vec<MessageElem>>>(
        &mut self,
        image: T,
        flash: bool,
    ) {
        if flash {
            self.results.chain.push::<FlashImage>(image.into())
        } else {
            self.results.chain.push(image)
        }
    }
    pub(crate) async fn push_image(&mut self, image: Images, flash: bool) -> Result<(), RespError> {
        if self.group {
            if let Some(image) = image
                .try_into_group_elem(self.cli, self.target, self.data_path)
                .await
            {
                Ok(self.push_flash(image, flash))
            } else {
                Err(error::rq_error("upload group image failed"))
            }
        } else {
            if let Some(image) = image
                .try_into_friend_elem(self.cli, self.target, self.data_path)
                .await
            {
                Ok(self.push_flash(image, flash))
            } else {
                Err(error::rq_error("upload friend image failed"))
            }
        }
    }
    pub(crate) async fn push_image_data(
        &mut self,
        data: Vec<u8>,
        flash: bool,
    ) -> Result<(), RespError> {
        if self.group {
            match self.cli.upload_group_image(self.target, data).await {
                Ok(image) => Ok(self.push_flash(image, flash)),
                Err(e) => {
                    warn!(target: crate::WALLE_Q, "群图片上传失败：{}", e);
                    Err(error::rq_error(e))
                }
            }
        } else {
            match self.cli.upload_friend_image(self.target, data).await {
                Ok(image) => Ok(self.push_flash(image, flash)),
                Err(e) => {
                    warn!(target: crate::WALLE_Q, "好友图片上传失败：{}", e);
                    Err(error::rq_error(e))
                }
            }
        }
    }
}

pub(crate) fn rq_elem2msg_seg(elem: RQElem, wqdb: &WQDatabase) -> Option<MsgSegment> {
    match elem {
        RQElem::Text(text) => Some(MsgSegment {
            ty: "text".to_string(),
            data: value_map! {"text": text.content},
        }),
        RQElem::At(elem::At { target: 0, .. }) => Some(MsgSegment {
            ty: "mention_all".to_string(),
            data: value_map! {},
        }),
        RQElem::At(at) => Some(MsgSegment {
            ty: "mention".to_string(),
            data: value_map! {"user_id": at.target.to_string()},
        }),
        RQElem::Face(face) => Some(MsgSegment {
            ty: "face".to_owned(),
            data: value_map! {
                "id": face.index,
                "file": face.name
            },
        }),
        RQElem::MarketFace(face) => Some(MsgSegment {
            ty: "text".to_string(),
            data: value_map! {"text": face.name},
        }),
        RQElem::Dice(d) => Some(MsgSegment {
            ty: "dice".to_string(),
            data: value_map! {
                "value": d.value,
            },
        }),
        RQElem::FingerGuessing(f) => Some(MsgSegment {
            ty: "rps".to_string(),
            data: value_map! {
                "value": match f {
                    elem::FingerGuessing::Rock => 0i8,
                    elem::FingerGuessing::Paper => 1,
                    elem::FingerGuessing::Scissors => 2,
                }
            },
        }),
        RQElem::LightApp(l) => Some(MsgSegment {
            ty: "json".to_string(),
            data: value_map! {"data": l.content},
        }),
        RQElem::FriendImage(i) => {
            wqdb.insert_image(&i);
            Some(MsgSegment {
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
            Some(MsgSegment {
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
                Some(MsgSegment {
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
                Some(MsgSegment {
                    ty: "image".to_string(),
                    data: value_map! {
                        "file_id": gi.hex_image_id(),
                        "url": gi.url(),
                        "flash": true
                    },
                })
            }
        },
        RQElem::RichMsg(rich) => Some(MsgSegment {
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

pub(crate) fn msg_chain2msg_seg_vec(chain: MessageChain, wqdb: &WQDatabase) -> Vec<MsgSegment> {
    let mut rv = vec![];
    if let Some(reply) = chain.reply() {
        rv.push(
            segment::Reply {
                message_id: reply.reply_seq.to_string(),
                user_id: Some(reply.sender.to_string()),
            }
            .into(),
        )
    }
    for seg in chain.into_iter().filter_map(|s| rq_elem2msg_seg(s, wqdb)) {
        rv.push(seg);
    }
    rv
}
