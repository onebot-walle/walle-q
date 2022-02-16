use walle_core::{BaseEvent, Event, MessageContent};

pub(crate) mod sled;

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database: DatabaseInit + Sized {
    fn get_message_event(&self, key: &str) -> Option<BaseEvent<MessageContent>>;
    fn insert_message_event(&self, value: &Event);
    fn get_latest_message_events(&self, limit: usize) -> Vec<Event>;
}
