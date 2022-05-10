use ricq::handler::QEvent;

pub async fn meta_event_process(ob: &walle_v11::impls::OneBot11, event: &QEvent) {
    match event {
        QEvent::Login(uin) => *ob.self_id.write().await = uin.to_string(),
        _ => {
            //todo
        }
    }
}
