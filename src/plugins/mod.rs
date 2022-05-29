mod executor;
mod gift_thanker;

use crate::client::BiliMessage;

pub trait DanmujiPlugin: Send + 'static {
    // asynchrounos function that accepts and processes a BiliMessage
    fn process_mesage(&mut self, msg: &BiliMessage) -> Option<String>;
}

pub use executor::DanmujiPluginExecutor;
pub use gift_thanker::GiftThanker;
