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
    error,
    model::{GroupTemp, Names, UserName, WalleQ, QQ},
};

pub(crate) async fn new_event<T, D, S, P, I>(time: Option<f64>, content: (T, D, S, P, I)) -> Event
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

pub(crate) fn new_group_msg_content(
    group_message: GroupMessage,
    message: Segments,
    selft: Selft,
) -> (Message, Group, (), Names, WalleQ) {
    (
        Message {
            selft,
            message_id: new_group_message_id(
                group_message.group_code,
                group_message.seqs,
                group_message.rands,
            ),
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
    )
}

pub(crate) async fn new_group_receipt_content(
    cli: &Client,
    receipt: MessageReceipt,
    group_code: i64,
    message: Segments,
    selft: Selft,
) -> (Message, Group, (), QQ, WalleQ) {
    (
        Message {
            selft,
            message_id: new_group_message_id(group_code, receipt.seqs, receipt.rands),
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
    )
}

pub(crate) fn new_group_audio_content(
    group_audio: GroupAudioMessage,
    message: Segments,
    selft: Selft,
) -> (Message, Group, (), Names, WalleQ) {
    (
        Message {
            selft,
            message_id: new_group_message_id(
                group_audio.group_code,
                group_audio.seqs,
                group_audio.rands,
            ),
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
    )
}

pub(crate) fn new_private_msg_content(
    friend_message: FriendMessage,
    message: Segments,
    selft: Selft,
) -> (Message, Private, (), UserName, WalleQ) {
    (
        Message {
            selft,
            alt_message: alt_message(&message),
            message,
            message_id: new_private_message_id(
                friend_message.from_uin,
                friend_message.time,
                friend_message.seqs,
                friend_message.rands,
            ),
            user_id: friend_message.from_uin.to_string(),
        },
        Private {},
        (),
        UserName {
            user_name: friend_message.from_nick.to_string(),
        },
        WalleQ {},
    )
}

pub(crate) async fn new_private_receipt_content(
    cli: &Client,
    receipt: MessageReceipt,
    target_id: i64,
    message: Segments,
    selft: Selft,
) -> (Message, Private, (), UserName, WalleQ) {
    (
        Message {
            selft,
            alt_message: alt_message(&message),
            message,
            message_id: new_private_message_id(
                target_id,
                receipt.time as i32,
                receipt.seqs,
                receipt.rands,
            ),
            user_id: cli.uin().await.to_string(),
        },
        Private {},
        (),
        UserName {
            user_name: cli.account_info.read().await.nickname.clone(),
        },
        WalleQ {},
    )
}

pub(crate) fn new_private_audio_content(
    friend_audio: FriendAudioMessage,
    message: Segments,
    selft: Selft,
) -> (Message, Private, (), UserName, WalleQ) {
    (
        Message {
            selft,
            alt_message: alt_message(&message),
            message,
            message_id: new_private_message_id(
                friend_audio.from_uin,
                friend_audio.time,
                friend_audio.seqs,
                friend_audio.rands,
            ),
            user_id: friend_audio.from_uin.to_string(),
        },
        Private {},
        (),
        UserName {
            user_name: friend_audio.from_nick.to_string(),
        },
        WalleQ {},
    )
}

pub(crate) fn new_group_temp_msg_content(
    group_temp: GroupTempMessage,
    message: Segments,
    selft: Selft,
) -> (Message, GroupTemp, (), UserName, WalleQ) {
    (
        Message {
            selft,
            alt_message: alt_message(&message),
            message,
            message_id: new_private_message_id(
                group_temp.from_uin,
                group_temp.time,
                group_temp.seqs,
                group_temp.rands,
            ),
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
    )
}

pub(crate) async fn new_group_temp_receipt_content(
    receipt: MessageReceipt,
    message: Segments,
    cli: &Client,
    group_code: i64,
    target_id: i64,
    selft: Selft,
) -> (Message, GroupTemp, (), UserName, WalleQ) {
    (
        Message {
            selft,
            alt_message: alt_message(&message),
            message,
            message_id: new_private_message_id(
                target_id,
                receipt.time as i32,
                receipt.seqs,
                receipt.rands,
            ),
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
    )
}
