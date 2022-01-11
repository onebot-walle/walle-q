use super::Database;
use sled::Db;
use walle_core::Event;

pub(crate) type SledDb = Db;

impl Database for Db {
    fn init() -> Self {
        sled::open("./data/sled").unwrap()
    }

    fn get_event(&self, key: &str) -> Option<Event> {
        self.get(key)
            .unwrap()
            .map(|v| rmp_serde::from_read(v.as_ref()).unwrap())
    }

    fn insert_event(&self, value: &Event) {
        self.insert(&value.id, rmp_serde::to_vec(value).unwrap())
            .unwrap();
    }
}

#[test]
fn sled_test() {
    use walle_core::{resp::StatusContent, BaseEvent, EventContent, MetaContent};
    let db = SledDb::init();
    db.insert_event(&BaseEvent {
        id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
        r#impl: "rs_onebot_qq".to_owned(),
        platform: "qq".to_owned(),
        self_id: "123234".to_owned(),
        time: 1632847927,
        content: EventContent::Meta(MetaContent::Heartbeat {
            interval: 5000,
            status: StatusContent {
                good: true,
                online: true,
            },
            sub_type: "".to_owned(),
        }),
    });
    println!(
        "{:?}",
        db.get_event("b6e65187-5ac0-489c-b431-53078e9d2bbb")
    );
}

#[test]
fn sled_get_test() {
    let db = SledDb::init();
    println!(
        "{:?}",
        db.get_event("b6e65187-5ac0-489c-b431-53078e9d2bbb")
    );
}
