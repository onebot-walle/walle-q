use crate::database::Database;
use crate::parse::Parse;
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use walle_core::{
    action::SendMessageContent, impls::OneBot, resp::SendMessageRespContent, Action, ActionHandler,
    MessageContent, Resps,
};

pub(crate) struct AHandler(pub(crate) Arc<rs_qq::Client>);

#[async_trait]
impl ActionHandler<Action, Resps, OneBot> for AHandler {
    async fn handle(&self, action: Action, ob: &OneBot) -> Result<Resps, Resps> {
        match action {
            Action::SendMessage(msg) => self.handle(msg, ob).await,
            _ => Err(Resps::unsupported_action()),
        }
    }
}

#[async_trait]
impl ActionHandler<SendMessageContent, Resps, OneBot> for AHandler {
    async fn handle(&self, content: SendMessageContent, ob: &OneBot) -> Result<Resps, Resps> {
        let group_id = content.group_id.ok_or(Resps::bad_param())?;
        if &content.detail_type == "group" {
            self.0
                .send_group_message(
                    group_id.parse().map_err(|_| Resps::bad_param())?,
                    content.message.clone().parse(),
                )
                .await
                .map_err(|_| Resps::platform_error())?;
            let event = ob.new_event(
                MessageContent::new_group_message_content(
                    content.message,
                    ob.self_id.read().await.clone(),
                    group_id,
                    HashMap::new(),
                )
                .into(),
            ).await;
            crate::SLED_DB.insert_event(&event);
            Ok(Resps::success(
                SendMessageRespContent {
                    message_id: event.id,
                    time: event.time,
                }
                .into(),
            ))
        } else if &content.detail_type == "private" {
            Err(Resps::unsupported_action())
        } else {
            Err(Resps::unsupported_action())
        }
    }
}
