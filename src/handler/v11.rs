use std::sync::Arc;

use walle_core::ActionHandler;
use walle_v11::{impls::OneBot11, Action, Resp};

pub(crate) struct V11Handler(pub Arc<walle_core::impls::OneBot>);

#[async_trait::async_trait]
impl ActionHandler<Action, Resp, OneBot11> for V11Handler {
    async fn handle(&self, action: Action, _: &OneBot11) -> Resp {
        let v12_action: Result<walle_core::StandardAction, _> = action.try_into();
        if let Ok(a) = v12_action {
            self.0.action_handler.handle(a, &self.0).await.into()
        } else {
            Resp::empty_404()
        }
    }
}
