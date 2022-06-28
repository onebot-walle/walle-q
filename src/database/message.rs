use crate::extra::{WQEvent, WQEventContent};

pub trait MessageId {
    fn message_id(&self) -> String;
}

impl MessageId for WQEvent {
    fn message_id(&self) -> String {
        if let WQEventContent::Message(ref c) = self.content {
            c.message_id.clone()
        } else {
            String::default()
        }
    }
}
