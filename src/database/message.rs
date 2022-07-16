use walle_core::{event::Event, util::ValueMapExt};

pub trait MessageId {
    fn message_id(&self) -> String;
}

impl MessageId for Event {
    fn message_id(&self) -> String {
        self.extra.get_downcast("message_id").unwrap_or_default()
    }
}
