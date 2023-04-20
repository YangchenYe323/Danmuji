mod context;

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use tokio::sync::{broadcast::Receiver, mpsc::UnboundedSender};
use tracing::error;

use crate::{client::BiliMessage, DanmujiResult};

use self::context::ChatbotMessageBuilder;

const MAX_TOKEN: u16 = 4096;
const MAX_COMPLETION_TOKEN: u16 = 1024;
const MAX_MESSAGE_TOKEN: u16 = MAX_TOKEN - MAX_COMPLETION_TOKEN;
const PERSIST_TO: &str = "gpt.data";

#[derive(Debug)]
pub struct Chatbot {
  shutdown: Arc<AtomicBool>,
}

impl Chatbot {
  pub fn start(
    upstream: Receiver<BiliMessage>,
    downstream: UnboundedSender<String>,
  ) -> DanmujiResult<Self> {
    let context = ChatbotMessageBuilder::new(MAX_MESSAGE_TOKEN, PERSIST_TO)?;
    let bot = Self {
      shutdown: Arc::new(AtomicBool::new(false)),
    };

    tokio::spawn(start_bot(
      bot.shutdown.clone(),
      upstream,
      downstream,
      context,
    ));

    Ok(bot)
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
  mut context: ChatbotMessageBuilder,
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
    if let Some(content) = content.strip_prefix(IDENTIFIER) {
      context.add_request_message(content, comment.uname());
      let request_messages = context.get_request_messages();
      let request = async_openai::types::CreateChatCompletionRequestArgs::default()
        .max_tokens(MAX_COMPLETION_TOKEN)
        .model("gpt-3.5-turbo")
        .messages(request_messages)
        .build()
        .unwrap();
      let res = client.chat().create(request).await;
      match res {
        Ok(response) => {
          for choice in response.choices {
            context.add_response_message(&choice.message);
            if let Err(err) = downstream.send(choice.message.content) {
              error!("Danmu Sender Dropped: {}", err);
              break;
            }
          }
        }
        Err(err) => {
          error!("{}", err);
        }
      }
    }
  }
}
