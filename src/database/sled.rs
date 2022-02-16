use super::{Database, DatabaseInit};
use sled::Db;
use walle_core::{Event, EventContent};

pub(crate) type SledDb = Db;

impl DatabaseInit for Db {
    fn init() -> Self {
        sled::open("./data/sled").unwrap()
    }
}

impl Database for Db {
    fn get_message_event(&self, key: &str) -> Option<Event> {
        self.get(key)
            .unwrap()
            .map(|v| rmp_serde::from_read(v.as_ref()).unwrap())
    }

    fn insert_message_event(&self, value: &Event) {
        if let EventContent::Message(ref m) = value.content {
            self.insert(&m.message_id, rmp_serde::to_vec(value).unwrap())
                .unwrap();
        }
    }
}

#[test]
fn sled_test() {
    use walle_core::{resp::StatusContent, BaseEvent, Event, EventContent, MetaContent};
    let db = SledDb::init();
    db.insert_message_event(
        &BaseEvent {
            id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
            r#impl: "rs_onebot_qq".to_owned(),
            platform: "qq".to_owned(),
            self_id: "123234".to_owned(),
            time: 1632847927.0,
            content: EventContent::Meta(MetaContent::Heartbeat {
                interval: 5000,
                status: StatusContent {
                    good: true,
                    online: true,
                },
                sub_type: "".to_owned(),
            }),
        },
    );
    let e: Option<Event> = db.get_message_event("b6e65187-5ac0-489c-b431-53078e9d2bbb");
    println!("{:?}", e);
}

#[test]
fn sled_get_test() {
    use walle_core::Event;
    let db = SledDb::init();
    let e: Option<Event> = db.get_message_event("b6e65187-5ac0-489c-b431-53078e9d2bbb");
    println!("{:?}", e);
}
