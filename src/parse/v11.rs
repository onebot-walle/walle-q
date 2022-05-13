use ricq::handler::QEvent;

pub async fn meta_event_process<H>(ob: &walle_v11::impls::OneBot11<H>, event: &QEvent) {
    match event {
        QEvent::Login(uin) => *ob.self_id.write().await = uin.to_string(),
        _ => {
            //todo
        }
    }
}
