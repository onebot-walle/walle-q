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
pub use segment::*;
pub use values::*;

use walle_core::prelude::{PushToValueMap, ToEvent};

#[derive(Debug, Clone, Copy, ToEvent, PushToValueMap)]
#[event(impl = "walle-q")]
pub struct WalleQ;

#[derive(Debug, Clone, Copy, ToEvent, PushToValueMap)]
#[event(platform = "qq")]
pub struct QQ;
