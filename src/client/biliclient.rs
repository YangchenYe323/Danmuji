//! This module implements the BiliClient type,
//! which wraps around rust-websocket with specific support
//! for Bilibili live's protocol behavior and message encoding/decoding

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use tracing::{error, info};
use websocket::Message;

use crate::DanmujiResult;

use super::{common::BiliMessage, message::BiliWebsocketMessage};

// Bilibili's Websocket URL
const URL: &'static str = "ws://broadcastlv.chat.bilibili.com:2244/sub";

// consumer type
pub type Consumer = Sender<BiliMessage>;

/// The wrapper type around rust's websocket client
/// functionality, that adds specific support for Bilibili's
/// websocket API
pub struct BiliClient {
    // control flag
    shutdown: Arc<AtomicBool>,
    // downstream consumer of the messages we create
    downstream: Arc<Mutex<Option<Consumer>>>,

    worker: Option<JoinHandle<()>>,
}

impl BiliClient {
    /// start a [BiliClient] instance that connects to @roomid
    /// as the user of @userid
    pub fn start(room_id: i64, user_id: Option<u64>) -> DanmujiResult<Self> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let downstream = Arc::new(Mutex::new(None));

        let config = ClientConfig {
            room_id,
            user_id,
            shutdown: shutdown.clone(),
            downstream: downstream.clone(),
        };

        let worker = std::thread::spawn(move || start_worker(config));

        Ok(Self {
            shutdown,
            downstream,
            worker: Some(worker),
        })
    }

    /// set up downstream consumer
    pub fn set_downstream(&self, downstream: Consumer) {
        *self.downstream.lock().unwrap() = Some(downstream);
    }

    /// shutdown this client
    /// return the downstream consumer so that it can be plugged
    /// into other producers in the future
    pub fn shutdown(self) -> Option<Consumer> {
        self.downstream.lock().unwrap().take()
        // self dropped here
    }
}

// impl Drop for BiliClient {
// 	fn drop(&mut self) {
// 		self.shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
// 		// let thread be cleaned up
// 		let worker = self.worker.take();
// 		if let Some(worker) = worker {
// 			worker.join().unwrap();
// 			info!("BiliClient Worker Thread Collected, Termintating...")
// 		}
// 	}
// }

#[derive(Debug, Clone)]
struct ClientConfig {
    room_id: i64,
    user_id: Option<u64>,
    // shared with the top-level Client Handle & will be modified
    // by the handle to signal termination
    shutdown: Arc<AtomicBool>,
    // shared with the top-level Client Handle & will be set
    // by the handle to signal termination
    downstream: Arc<Mutex<Option<Consumer>>>,
}

/// The result of a client's worker thread
#[derive(Debug)]
enum ClientResult {
    // Client is terminated by the user
    // Just clean up and return
    Terminated,
    // Client's connection is accidentally closed
    // by the server, try to re-run another connection
    LostConnection,
}

fn start_worker(config: ClientConfig) {
    loop {
        let cfg = config.clone();
        let worker_handle = std::thread::spawn(move || -> DanmujiResult<ClientResult> {
            let ClientConfig {
                room_id,
                user_id,
                shutdown,
                downstream,
            } = cfg;

            let cli = websocket::ClientBuilder::new(URL)
                .unwrap()
                .connect_insecure()?;

            let (mut reader, mut writer) = cli.split()?;

            // send entry data
            let entry_msg = BiliWebsocketMessage::entry(room_id, user_id);
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

                    let res = writer.send_message(&Message::binary(heartbeat.to_vec()));

                    if res.is_err() {
                        debug!("HeartBeat Thread Error {:?}", res);
                        break;
                    }

                    std::thread::sleep(Duration::from_secs(20));
                }

                info!("Heart Beat Thread Shutdown");
            });

            // reader
            let result = loop {
                if shutdown.load(Ordering::Relaxed) {
                    break ClientResult::Terminated;
                }
                let message = reader.recv_message();
                match message {
                    Ok(message) => {
                        match message {
                            websocket::OwnedMessage::Binary(buf) => {
                                let msg = BiliWebsocketMessage::from_binary(buf).unwrap();

                                for inner in msg.parse() {
                                    let bili_msg = BiliMessage::from_raw_wesocket_message(inner);
                                    if let Some(msg) = bili_msg {
                                        info!("Received Msg: {:?}", msg);
                                        // send message to downstream
                                        {
                                            let downstream_guard = downstream.lock().unwrap();
                                            if let Some(guard) = downstream_guard.as_ref() {
                                                let res = guard.send(msg);
                                                if let Err(err) = res {
                                                    error!("{}", err);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            websocket::OwnedMessage::Close(_) => {
                                break ClientResult::LostConnection;
                            }

                            _ => {
                                info!("{:?}", message);
                            }
                        }
                    }
                    // websocket error
                    Err(err) => {
                        error!("{}", err);
                        break ClientResult::LostConnection;
                    }
                }
            };

            Ok(result)
        });

        // during normal execution, worker_handle runs
        // indefinitely until either: (a). shutdown flag is set externally
        // or (b). Connection closed by Server
        // Both cases are handled below
        let result = worker_handle.join().unwrap();

        match result {
            Ok(ClientResult::Terminated) => break,

            Ok(ClientResult::LostConnection) => {
                info!("Websocket connection lost by accident, Reconnecting...");
                continue;
            }

            Err(err) => {
                error!("Websocket Error: {}, try reconnecting..", err);
                continue;
            }
        }
    }

    // terminated
    assert!(config.shutdown.load(Ordering::Relaxed));
    info!("Websocket terminated");
}
