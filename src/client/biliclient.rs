use std::{
	sync::{
		Arc,
		atomic::AtomicBool, Mutex, mpsc::Receiver,
	}, time::Duration
};

use tracing::info;
use websocket::Message;

use crate::{Result, core::message::BiliWebsocketMessage};

use super::common::BiliMessage;

const URL: &'static str = "ws://broadcastlv.chat.bilibili.com:2244/sub";

/// The wrapper type around rust's websocket client 
/// functionality, that adds specific support for Bilibili's
/// websocket API
pub struct BiliClient {
	// control flag
	shutdown: Arc<AtomicBool>,
	// downstream consumer of the messages we create
	downstream: Arc<Mutex<Option<Receiver<BiliMessage>>>>,
}

impl BiliClient {
	/// start a [BiliClient] instance that connects to @roomid
	/// as the user of @userid
	pub fn start(roomid: i64, userid: Option<u64>) -> Result<Self> {
		let shutdown = Arc::new(AtomicBool::new(false));
		let downstream = Arc::new(Mutex::new(None));

		let cli = websocket::ClientBuilder::new(URL)
			.unwrap()
			.connect_insecure()?;
		
		info!("Connection to Websocket Server established");

		let (mut reader, mut writer) = cli.split()?;
		// send entry data
		let entry_msg = BiliWebsocketMessage::entry(roomid, userid);
		writer.send_message(&Message::binary(entry_msg.to_vec()))?;

		let st = Arc::clone(&shutdown);
		// start heartbeat thread
		std::thread::spawn(move || {
			// send heart beat
			info!("Heart Beat Thread Started Running");
			loop {

				if st.load(std::sync::atomic::Ordering::Relaxed) {
					break;
				}

				let heartbeat = BiliWebsocketMessage::heartbeat();
				writer
					.send_message(&Message::binary(heartbeat.to_vec()))
					.unwrap();
				std::thread::sleep(Duration::from_secs(20));
			}

			info!("Heart Beat Thread Shutdown");
		});

		let st = Arc::clone(&shutdown);
		let consumer = Arc::clone(&downstream);
		// start message thread
		std::thread::spawn(move || {
			for message in reader.incoming_messages() {
				// check shutdown
				if st.load(std::sync::atomic::Ordering::Relaxed) {
					break;
				}

				match message {
					Ok(message) => {
						match message {
							websocket::OwnedMessage::Binary(buf) => {
								let msg = BiliWebsocketMessage::from_binary(buf).unwrap();
								
								for inner in msg.parse() {
									info!("Received Message: {:?}", inner);
									// send to consumer
								}
							}
		
							_ => {
								info!("{:?}", message);
							}
						}
					}
					// websocket is closed
					Err(websocket::WebSocketError::NoDataAvailable) => {
						warn!("Websocket is closed by server");
						break;
					}
					//todo: don't know how to handle the other errors
					_ => continue,
				}
            }

			info!("Message Thread Shut down");
		});

		Ok(Self {
			shutdown,
			downstream,
		})
	}

	/// set up downstream consumer
	pub fn set_downstream(&self, downstream: Receiver<BiliMessage>) {
		*self.downstream.lock().unwrap() = Some(downstream);
	}

	/// shutdown this client
	/// return the downstream consumer so that it can be plugged
	/// into other producers in the future
	pub fn shutdown(self) -> Option<Receiver<BiliMessage>>{
		self.downstream.lock().unwrap().take()
		// self dropped here
	}
	
}

// impl Drop for BiliClient {
// 	fn drop(&mut self) {
// 		self.shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
// 		// let thread be cleaned up
// 	}
// }