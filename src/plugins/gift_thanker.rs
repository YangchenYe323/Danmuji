use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use serde::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;
use tokio::sync::{broadcast::Receiver, mpsc::UnboundedSender, Mutex};
use tracing::error;
use ts_rs::TS;

use crate::client::BiliMessage;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/GiftThankConfig.ts")]
pub struct GiftThankConfig {
  // reply template
  template: String,
  // open or closed
  open: bool,
}

impl Default for GiftThankConfig {
  fn default() -> Self {
    Self {
      template: "感谢{uname}投喂的{gift_num}个{gift_name}~".to_string(),
      open: true,
    }
  }
}

impl GiftThankConfig {
  pub fn get_thank_message(&self, msg: &BiliMessage) -> Option<String> {
    if !self.open {
      return None;
    }

    match msg {
      BiliMessage::Gift(ref gift) => {
        let mut template = TinyTemplate::new();
        if let Err(err) = template.add_template("gift", &self.template) {
          error!("Invalid Gift Thank Template: {}", err);
          None
        } else if let Ok(res) = template.render("gift", gift) {
          Some(res)
        } else {
          None
        }
      }
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct GiftThanker {
  shutdown: Arc<AtomicBool>,
  config: Arc<Mutex<Option<GiftThankConfig>>>,
}

impl GiftThanker {
  pub fn start(
    config: GiftThankConfig,
    upstream: Receiver<BiliMessage>,
    downstream: UnboundedSender<String>,
  ) -> Self {
    let thanker = Self {
      shutdown: Arc::new(AtomicBool::new(false)),
      config: Arc::new(Mutex::new(Some(config))),
    };

    tokio::spawn(start_thanker(
      thanker.shutdown.clone(),
      upstream,
      thanker.config.clone(),
      downstream,
    ));

    thanker
  }

  pub async fn get_config(&self) -> Option<GiftThankConfig> {
    self.config.lock().await.clone()
  }

  pub async fn set_config(&self, config: GiftThankConfig) {
    *self.config.lock().await = Some(config);
  }
}

impl Drop for GiftThanker {
  fn drop(&mut self) {
    self.shutdown.store(true, Ordering::Relaxed);
  }
}

async fn start_thanker(
  shutdown: Arc<AtomicBool>,
  mut upstream: Receiver<BiliMessage>,
  config: Arc<Mutex<Option<GiftThankConfig>>>,
  downstream: UnboundedSender<String>,
  // plugins: Arc<Mutex<HashMap<&'static str, Box<dyn DanmujiPlugin>>>>,
) {
  loop {
    if shutdown.load(Ordering::Relaxed) {
      break;
    }

    let msg = upstream.recv().await;
    if let Err(err) = msg {
      error!("BiliClient dropped: {}", err);
      break;
    }

    let msg = msg.unwrap();
    {
      let config = config.lock().await;
      if let Some(config) = config.as_ref() {
        let reply = config.get_thank_message(&msg);
        if let Some(reply) = reply {
          if let Err(err) = downstream.send(reply) {
            error!("Danmu Sender Dropped: {}", err);
            break;
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::client::GiftMessage;

  #[tokio::test]
  async fn test_template_basics() {
    let test_msg = BiliMessage::Gift(GiftMessage::default_message());
    let config: GiftThankConfig = Default::default();
    assert_eq!(
      Some("感谢测试用户投喂的1个小花花~".to_string()),
      config.get_thank_message(&test_msg)
    );
  }
}
