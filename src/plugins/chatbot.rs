use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use async_openai::types::Role;
use tokio::sync::{broadcast::Receiver, mpsc::UnboundedSender};
use tracing::error;

use crate::client::BiliMessage;

#[derive(Debug)]
pub struct Chatbot {
  shutdown: Arc<AtomicBool>,
}

impl Chatbot {
  pub fn start(upstream: Receiver<BiliMessage>, downstream: UnboundedSender<String>) -> Self {
    let bot = Self {
      shutdown: Arc::new(AtomicBool::new(false)),
    };

    tokio::spawn(start_bot(bot.shutdown.clone(), upstream, downstream));

    bot
  }
}

impl Drop for Chatbot {
  fn drop(&mut self) {
    self.shutdown.store(true, Ordering::Relaxed);
  }
}

async fn start_bot(
  shutdown: Arc<AtomicBool>,
  mut upstream: Receiver<BiliMessage>,
  downstream: UnboundedSender<String>,
) {
  const IDENTIFIER: &str = "@bot ";
  let client = async_openai::Client::new();
  loop {
    if shutdown.load(Ordering::Relaxed) {
      break;
    }
    let msg = upstream.recv().await;
    if let Err(err) = msg {
      error!("BiliClient dropped: {}", err);
      break;
    };
    let msg = msg.unwrap();
    let BiliMessage::Danmu(comment) = msg else {
      continue;
    };
    let content = comment.content();
    if content.starts_with(IDENTIFIER) {
      let mut split = content.split(IDENTIFIER);
      split.next();
      let content = split.next().unwrap().trim();
      let request = async_openai::types::CreateChatCompletionRequestArgs::default()
        .max_tokens(1024u16)
        .model("gpt-3.5-turbo")
        .messages([
          async_openai::types::ChatCompletionRequestMessageArgs::default()
            .content(content)
            .role(Role::User)
            .build()
            .unwrap(),
        ])
        .build()
        .unwrap();
      let response = client.chat().create(request).await.unwrap();
      for choice in response.choices {
        if let Err(err) = downstream.send(choice.message.content) {
          error!("Danmu Sender Dropped: {}", err);
          break;
        }
      }
    }
  }
}
