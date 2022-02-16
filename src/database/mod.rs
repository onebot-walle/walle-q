use walle_core::Event;

pub(crate) mod sled;

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database: DatabaseInit + Sized {
    fn get_message_event(&self, key: &str) -> Option<Event>;
    fn insert_message_event(&self, value: &Event);
}
