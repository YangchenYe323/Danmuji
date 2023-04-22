use std::{
  collections::{HashMap, VecDeque},
  fs::{File, OpenOptions},
  io::Write,
  path::Path,
};

use async_openai::types::{
  ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, ChatCompletionResponseMessage,
  Role,
};
use tracing::error;

use crate::DanmujiResult;

#[derive(Debug)]
pub struct ChatbotMessageBuilder {
  message_log: VecDeque<ChatCompletionRequestMessage>,
  id_generator: UserIdGenerator,
  persister: FilePersister,
}

impl ChatbotMessageBuilder {
  pub fn new(_max_token: u16, persist_to: impl AsRef<Path>) -> DanmujiResult<Self> {
    let persister = FilePersister::from_file(persist_to)?;
    Ok(Self {
      message_log: VecDeque::default(),
      id_generator: UserIdGenerator::default(),
      persister,
    })
  }

  pub fn add_request_message(&mut self, content: &str, user_name: &str) {
    let user_id = self.id_generator.generate(user_name);
    let msg = ChatCompletionRequestMessageArgs::default()
      .content(content)
      .role(Role::User)
      .name(user_id)
      .build()
      .unwrap();
    self.persister.persist_new_message(&msg);
    self.message_log.pop_front();
    self.message_log.push_back(msg);
  }

  pub fn add_response_message(&mut self, response: &ChatCompletionResponseMessage) {
    let msg = ChatCompletionRequestMessageArgs::default()
      .content(&response.content)
      .role(Role::System)
      .build()
      .unwrap();
    self.persister.persist_new_message(&msg);
    self.message_log.pop_front();
    self.message_log.push_back(msg);
  }

  pub fn get_request_messages(&mut self) -> &[ChatCompletionRequestMessage] {
    self.message_log.make_contiguous()
  }
}

#[derive(Debug)]
pub struct ChatbotMessagePersister<W: Write> {
  writer: W,
}

impl<W: Write> ChatbotMessagePersister<W> {
  pub fn new(writer: W) -> Self {
    Self { writer }
  }

  pub fn persist_new_message(&mut self, msg: &ChatCompletionRequestMessage) {
    if let Err(err) = serde_json::to_writer(&mut self.writer, msg) {
      error!("{}", err);
    }
  }
}

type FilePersister = ChatbotMessagePersister<File>;

impl FilePersister {
  pub fn from_file(path: impl AsRef<Path>) -> DanmujiResult<Self> {
    let file = OpenOptions::new().append(true).create(true).open(path)?;
    Ok(Self::new(file))
  }
}

#[derive(Debug, Default)]
pub struct UserIdGenerator {
  next_available: u32,
  ids: HashMap<String, String>,
}

impl UserIdGenerator {
  pub fn generate(&mut self, user_name: &str) -> &str {
    let key = user_name.to_string();
    self.ids.entry(key).or_insert_with(|| {
      let new_id = format!("User{}", self.next_available);
      self.next_available += 1;
      new_id
    })
  }
}
