use ricq::{
    structs::{
        FriendAudioMessage, FriendMessage, GroupAudioMessage, GroupMessage, GroupTempMessage,
        MessageReceipt,
    },
    Client,
};
use walle_core::{
    event::{
        new_event as _new_event, DetailTypeLevel, Event, Group, ImplLevel, Message, PlatformLevel,
        Private, SubTypeLevel, TypeLevel,
    },
    prelude::ToEvent,
    resp::RespError,
    segment::Segments,
    structs::Selft,
    util::{new_uuid, timestamp_nano_f64},
    value_map,
};

use crate::{
    database::{Database, WQDatabase},
    error,
    model::{GroupTemp, Names, UserName, WalleQ, QQ},
};

pub(crate) fn new_event<T, D, S, P, I>(time: Option<f64>, content: (T, D, S, P, I)) -> Event
where
    T: ToEvent<TypeLevel>,
    D: ToEvent<DetailTypeLevel>,
    S: ToEvent<SubTypeLevel>,
    P: ToEvent<PlatformLevel>,
    I: ToEvent<ImplLevel>,
{
    _new_event(
        new_uuid(),
        time.unwrap_or_else(timestamp_nano_f64),
        content.0,
        content.1,
        content.2,
        content.3,
        content.4,
        value_map!(),
    )
    .into()
}

fn alt_message(segemnts: &Segments) -> String {
    segemnts.iter().map(|seg| seg.alt()).collect()
}

pub(crate) fn new_group_message_id(group_code: i64, seqs: Vec<i32>, rands: Vec<i32>) -> String {
    [
        group_code.to_string(),
        seqs.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("-"),
        rands
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("-"),
    ]
    .join(" ")
}

pub(crate) fn new_private_message_id(
    uin: i64,
    time: i32,
    seqs: Vec<i32>,
    rands: Vec<i32>,
) -> String {
    [
        uin.to_string(),
        seqs.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("-"),
        rands
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("-"),
        time.to_string(),
    ]
    .join(" ")
}

pub(crate) fn decode_message_id(
    message_id: &str,
) -> Result<(i64, Vec<i32>, Vec<i32>, Option<i32>), RespError> {
    let mut splits = message_id.split(' ');
    let target = if let Some(i) = splits.next() {
        i.parse()
            .map_err(|_| error::bad_param("message_id decode failed"))?
    } else {
        return Err(error::bad_param("message_id decode failed"));
    };
    let seqs = if let Some(i) = splits.next() {
        i.split('-')
            .map(|s| s.parse())
            .collect::<Result<Vec<i32>, _>>()
            .map_err(|_| error::bad_param("message_id decode failed"))?
    } else {
        return Err(error::bad_param("message_id decode failed"));
    };
    let rands = if let Some(i) = splits.next() {
        i.split('-')
            .map(|s| s.parse())
            .collect::<Result<Vec<i32>, _>>()
            .map_err(|_| error::bad_param("message_id decode failed"))?
    } else {
        return Err(error::bad_param("message_id decode failed"));
    };
    let time = if let Some(i) = splits.next() {
        Some(
            i.parse()
                .map_err(|_| error::bad_param("message_id decode failed"))?,
        )
    } else {
        None
    };
    Ok((target, seqs, rands, time))
}

pub(crate) fn new_group_msg(
    group_message: GroupMessage,
    message: Segments,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(group_message.time as f64),
        (
            Message {
                selft,
                message_id: if database.not_empty() {
                    group_message.seqs.first().unwrap().to_string()
                } else {
                    new_group_message_id(
                        group_message.group_code,
                        group_message.seqs.clone(),
                        group_message.rands.clone(),
                    )
                },
                alt_message: alt_message(&message),
                message,
                user_id: group_message.from_uin.to_string(),
            },
            Group {
                group_id: group_message.group_code.to_string(),
            },
            (),
            Names {
                group_name: group_message.group_name,
                user_name: group_message.group_card,
            },
            WalleQ {},
        ),
    );
    database.insert_message(&event, group_message.seqs, group_message.rands);
    event
}

pub(crate) async fn new_group_receipt(
    cli: &Client,
    receipt: MessageReceipt,
    group_code: i64,
    message: Segments,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(receipt.time as f64),
        (
            Message {
                selft,
                message_id: if database.not_empty() {
                    receipt.seqs.first().unwrap().to_string()
                } else {
                    new_group_message_id(group_code, receipt.seqs.clone(), receipt.rands.clone())
                },
                alt_message: alt_message(&message),
                message,
                user_id: cli.uin().await.to_string(),
            },
            Group {
                group_id: group_code.to_string(),
            },
            (),
            QQ {}, //todo
            WalleQ {},
        ),
    );
    database.insert_message(&event, receipt.seqs, receipt.rands);
    event
}

pub(crate) fn new_group_audio(
    group_audio: GroupAudioMessage,
    message: Segments,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(group_audio.time as f64),
        (
            Message {
                selft,
                message_id: if database.not_empty() {
                    group_audio.seqs.first().unwrap().to_string()
                } else {
                    new_group_message_id(
                        group_audio.group_code,
                        group_audio.seqs.clone(),
                        group_audio.rands.clone(),
                    )
                },
                alt_message: alt_message(&message),
                message,
                user_id: group_audio.from_uin.to_string(),
            },
            Group {
                group_id: group_audio.group_code.to_string(),
            },
            (),
            Names {
                group_name: group_audio.group_name.to_string(),
                user_name: group_audio.group_card.to_string(),
            },
            WalleQ {},
        ),
    );
    database.insert_message(&event, group_audio.seqs, group_audio.rands);
    event
}

pub(crate) fn new_private_msg(
    friend_message: FriendMessage,
    message: Segments,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(friend_message.time as f64),
        (
            Message {
                selft,
                alt_message: alt_message(&message),
                message,
                message_id: if database.not_empty() {
                    friend_message.seqs.first().unwrap().to_string()
                } else {
                    new_private_message_id(
                        friend_message.from_uin,
                        friend_message.time,
                        friend_message.seqs.clone(),
                        friend_message.rands.clone(),
                    )
                },
                user_id: friend_message.from_uin.to_string(),
            },
            Private {},
            (),
            UserName {
                user_name: friend_message.from_nick.to_string(),
            },
            WalleQ {},
        ),
    );
    database.insert_message(&event, friend_message.seqs, friend_message.rands);
    event
}

pub(crate) async fn new_private_receipt(
    cli: &Client,
    receipt: MessageReceipt,
    target_id: i64,
    message: Segments,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(receipt.time as f64),
        (
            Message {
                selft,
                alt_message: alt_message(&message),
                message,
                message_id: if database.not_empty() {
                    receipt.seqs.first().unwrap().to_string()
                } else {
                    new_private_message_id(
                        target_id,
                        receipt.time as i32,
                        receipt.seqs.clone(),
                        receipt.rands.clone(),
                    )
                },
                user_id: cli.uin().await.to_string(),
            },
            Private {},
            (),
            UserName {
                user_name: cli.account_info.read().await.nickname.clone(),
            },
            WalleQ {},
        ),
    );
    database.insert_message(&event, receipt.seqs, receipt.rands);
    event
}

pub(crate) fn new_private_audio(
    friend_audio: FriendAudioMessage,
    message: Segments,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(friend_audio.time as f64),
        (
            Message {
                selft,
                alt_message: alt_message(&message),
                message,
                message_id: if database.not_empty() {
                    friend_audio.seqs.first().unwrap().to_string()
                } else {
                    new_private_message_id(
                        friend_audio.from_uin,
                        friend_audio.time,
                        friend_audio.seqs.clone(),
                        friend_audio.rands.clone(),
                    )
                },
                user_id: friend_audio.from_uin.to_string(),
            },
            Private {},
            (),
            UserName {
                user_name: friend_audio.from_nick.to_string(),
            },
            WalleQ {},
        ),
    );
    database.insert_message(&event, friend_audio.seqs, friend_audio.rands);
    event
}

pub(crate) fn new_group_temp_msg(
    group_temp: GroupTempMessage,
    message: Segments,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(group_temp.time as f64),
        (
            Message {
                selft,
                alt_message: alt_message(&message),
                message,
                message_id: if database.not_empty() {
                    group_temp.seqs.first().unwrap().to_string()
                } else {
                    new_private_message_id(
                        group_temp.from_uin,
                        group_temp.time,
                        group_temp.seqs.clone(),
                        group_temp.rands.clone(),
                    )
                },
                user_id: group_temp.from_uin.to_string(),
            },
            GroupTemp {
                group_id: group_temp.group_code.to_string(),
            },
            (),
            UserName {
                user_name: group_temp.from_nick.to_string(),
            },
            WalleQ {},
        ),
    );
    database.insert_message(&event, group_temp.seqs, group_temp.rands);
    event
}

pub(crate) async fn new_group_temp_receipt(
    receipt: MessageReceipt,
    message: Segments,
    cli: &Client,
    group_code: i64,
    target_id: i64,
    selft: Selft,
    database: &WQDatabase,
) -> Event {
    let event = new_event(
        Some(receipt.time as f64),
        (
            Message {
                selft,
                alt_message: alt_message(&message),
                message,
                message_id: if database.not_empty() {
                    receipt.seqs.first().unwrap().to_string()
                } else {
                    new_private_message_id(
                        target_id,
                        receipt.time as i32,
                        receipt.seqs.clone(),
                        receipt.rands.clone(),
                    )
                },
                user_id: cli.uin().await.to_string(),
            },
            GroupTemp {
                group_id: group_code.to_string(),
            },
            (),
            UserName {
                user_name: cli.account_info.read().await.nickname.clone(),
            },
            WalleQ {},
        ),
    );
    database.insert_message(&event, receipt.seqs, receipt.rands);
    event
}
