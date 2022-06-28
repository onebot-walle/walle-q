use ricq::{
    structs::{
        FriendAudioMessage, FriendMessage, GroupAudioMessage, GroupMessage, GroupTempMessage,
        MessageReceipt,
    },
    Client,
};
use walle_core::{
    event::MessageContent,
    message::{Message, MessageAlt},
    resp::RespError,
    util::{new_uuid, timestamp_nano_f64},
};

use crate::{
    error,
    extra::{WQEvent, WQEventContent, WQMEDetail},
};

pub(crate) async fn new_event(cli: &Client, time: Option<f64>, content: WQEventContent) -> WQEvent {
    WQEvent {
        id: new_uuid(),
        r#impl: crate::WALLE_Q.to_string(),
        platform: crate::PLATFORM.to_string(),
        self_id: cli.uin().await.to_string(),
        time: time.unwrap_or_else(timestamp_nano_f64),
        content,
    }
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
    message: Message,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::Group {
            sub_type: "".to_string(),
            group_id: group_message.group_code.to_string(),
            group_name: group_message.group_name.to_string(),
            user_name: group_message.group_card.to_string(),
        },
        alt_message: message.alt(),
        message,
        message_id: new_group_message_id(
            group_message.group_code,
            group_message.seqs,
            group_message.rands,
        ),
        user_id: group_message.from_uin.to_string(),
    }
    .into()
}

pub(crate) async fn new_group_receipt_content(
    cli: &Client,
    receipt: MessageReceipt,
    group_code: i64,
    message: Message,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::Group {
            sub_type: "".to_string(),
            group_id: group_code.to_string(),
            group_name: "todo".to_string(), //todo
            user_name: cli.account_info.read().await.nickname.clone(), //todo
        },
        alt_message: message.alt(),
        message,
        message_id: new_group_message_id(group_code, receipt.seqs, receipt.rands),
        user_id: cli.uin().await.to_string(),
    }
    .into()
}

pub(crate) fn new_group_audio_content(
    group_audio: GroupAudioMessage,
    message: Message,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::Group {
            sub_type: "".to_string(),
            group_id: group_audio.group_code.to_string(),
            group_name: group_audio.group_name.to_string(),
            user_name: group_audio.group_card.to_string(),
        },
        alt_message: message.alt(),
        message,
        message_id: new_group_message_id(
            group_audio.group_code,
            group_audio.seqs,
            group_audio.rands,
        ),
        user_id: group_audio.from_uin.to_string(),
    }
    .into()
}

pub(crate) fn new_private_msg_content(
    friend_message: FriendMessage,
    message: Message,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::Private {
            sub_type: "".to_string(),
            user_name: friend_message.from_nick.to_string(),
        },
        alt_message: message.alt(),
        message,
        message_id: new_private_message_id(
            friend_message.from_uin,
            friend_message.time,
            friend_message.seqs,
            friend_message.rands,
        ),
        user_id: friend_message.from_uin.to_string(),
    }
    .into()
}

pub(crate) async fn new_private_receipt_content(
    cli: &Client,
    receipt: MessageReceipt,
    target_id: i64,
    message: Message,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::Private {
            sub_type: "".to_string(),
            user_name: cli.account_info.read().await.nickname.clone(),
        },
        alt_message: message.alt(),
        message,
        message_id: new_private_message_id(
            target_id,
            receipt.time as i32,
            receipt.seqs,
            receipt.rands,
        ),
        user_id: cli.uin().await.to_string(),
    }
    .into()
}

pub(crate) fn new_private_audio_content(
    friend_audio: FriendAudioMessage,
    message: Message,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::Private {
            sub_type: "".to_string(),
            user_name: friend_audio.from_nick.to_string(),
        },
        alt_message: message.alt(),
        message,
        message_id: new_private_message_id(
            friend_audio.from_uin,
            friend_audio.time,
            friend_audio.seqs,
            friend_audio.rands,
        ),
        user_id: friend_audio.from_uin.to_string(),
    }
    .into()
}

pub(crate) fn new_group_temp_msg_content(
    group_temp: GroupTempMessage,
    message: Message,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::GroupTemp {
            sub_type: "".to_string(),
            group_id: group_temp.group_code.to_string(),
            user_name: group_temp.from_nick.to_string(),
        },
        alt_message: message.alt(),
        message,
        message_id: new_private_message_id(
            group_temp.from_uin,
            group_temp.time,
            group_temp.seqs,
            group_temp.rands,
        ),
        //todo
        user_id: group_temp.from_uin.to_string(),
    }
    .into()
}

pub(crate) async fn new_group_temp_receipt_content(
    receipt: MessageReceipt,
    message: Message,
    cli: &Client,
    group_code: i64,
    target_id: i64,
) -> WQEventContent {
    MessageContent::<WQMEDetail> {
        detail: WQMEDetail::GroupTemp {
            sub_type: "".to_string(),
            group_id: group_code.to_string(),
            user_name: cli.account_info.read().await.nickname.clone(),
        },
        alt_message: message.alt(),
        message,
        message_id: new_private_message_id(
            target_id,
            receipt.time as i32,
            receipt.seqs,
            receipt.rands,
        ),
        //todo
        user_id: cli.uin().await.to_string(),
    }
    .into()
}
