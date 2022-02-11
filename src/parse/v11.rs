use super::{Parse, Parser};
use rs_qq::{
    handler::QEvent,
    msg::{elem::RQElem, MessageChain},
};
use walle_v11::{Event, MessageSegment};

impl Parse<Option<MessageSegment>> for RQElem {
    fn parse(self) -> Option<MessageSegment> {
        let msg_seg: Option<walle_core::MessageSegment> = self.parse();
        msg_seg.map(|seg| seg.try_into().ok()).flatten()
    }
}

impl Parse<Vec<MessageSegment>> for MessageChain {
    fn parse(self) -> Vec<MessageSegment> {
        self.into_iter().filter_map(|elem| elem.parse()).collect()
    }
}

impl Parse<MessageChain> for Vec<MessageSegment> {
    fn parse(self) -> MessageChain {
        let v12s: Vec<walle_core::MessageSegment> = self
            .into_iter()
            .map(|msg_seg| msg_seg.try_into().ok())
            .flatten()
            .collect();
        v12s.parse()
    }
}

#[async_trait::async_trait]
impl Parser<QEvent, Event> for walle_v11::impls::OneBot11 {
    async fn parse(&self, _event: QEvent) -> Option<Event> {
        todo!()
    }
}
