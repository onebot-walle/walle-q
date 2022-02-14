pub(crate) mod sled;

pub(crate) trait DatabaseInit {
    fn init() -> Self;
}

pub(crate) trait Database<E>: DatabaseInit + Sized {
    fn get_event(&self, key: &str) -> Option<E>;
    fn insert_event<T>(&self, key: T, value: &E)
    where
        T: ToString;
}
