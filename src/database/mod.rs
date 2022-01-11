pub(crate) mod sled;

pub(crate) trait Database: Sized {
    fn init() -> Self;
    fn get_event(&self, key: &str) -> Option<walle_core::Event>;
    fn insert_event(&self, value: &walle_core::Event);
}
