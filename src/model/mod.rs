mod action;
mod message;
mod notice;
mod request;
mod segment;
mod values;

pub use action::*;
pub use message::*;
pub use notice::*;
pub use request::*;
pub use values::*;
pub use segment::*;

use walle_core::prelude::{OneBot, PushToValueMap};

#[derive(Debug, Clone, Copy, OneBot, PushToValueMap)]
#[event(impl = "walle-q")]
pub struct WalleQ {}

#[derive(Debug, Clone, Copy, OneBot, PushToValueMap)]
#[event(platform = "qq")]
pub struct QQ {}
