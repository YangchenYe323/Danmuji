//! This module implements the [BiliClient] type,
//! [BiliClient] is the component in Danmuji that handles all the websocket
//! interaction with the Bilibili Servers. It's responsible for:
//! - Establishing websocket connections with BiliBili and keep it alive
//! - Converting raw websocket messages to our custom type
//! - Forwarding the converted structure to the downstream consumers

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::Stream;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::Error};
use tracing::{error, info, trace, warn};

use crate::DanmujiResult;

use super::{common::BiliMessage, message::BiliWebsocketMessage};

// Bilibili's Websocket URL
const URL: &str = "ws://broadcastlv.chat.bilibili.com:2244/sub";

// consumer type
pub type Consumer = tokio::sync::broadcast::Sender<BiliMessage>;

/// The [BiliClient] struct that represents a handle and manager to a
/// pool of background tasks that connect to & interact with BiliBili's
/// live room websocket servers.
///
/// It is constructed with a [Consumer] as downstream, which is currently a
/// broadcast Sender, where it will forward everything.
#[derive(Debug)]
pub struct BiliClient {
    // connected_room_id -> shutdown flag
    shutdown: HashMap<i64, Arc<AtomicBool>>,
    // connected_room_id -> JoinHandle that runs the connection
    tasks: HashMap<i64, tokio::task::JoinHandle<()>>,
    // downstream consumer of the client
    downstream: Consumer,
}

impl BiliClient {
    /// Create a Client instance and bind to the given consumer
    ///
    /// * `downstream` downstream consumer of the messages
    pub fn new(downstream: Consumer) -> Self {
        Self {
            shutdown: HashMap::new(),
            tasks: HashMap::new(),
            downstream,
        }
    }

    /// Start a [BiliClient] instance that connects to specified room
    /// as the given user.
    /// This method can safely be called many times to connect to multiple live rooms.
    ///
    /// * `room_id` id of the connected room
    /// * `user_id` id of user, 0 if not provided
    ///
    /// # Note:
    /// When user_id is 0, websocket sometimes fails to receive meaningful messages, so it
    /// is recommended that a valid user_id be provided
    pub fn start(&mut self, room_id: i64, user_id: Option<u64>) -> DanmujiResult<()> {
        if self.shutdown.contains_key(&room_id) {
            return Ok(());
        }

        // the control signal for the room_id
        let shutdown = Arc::new(AtomicBool::new(false));
        let task = {
            let shutdown = shutdown.clone();
            let downstream = self.downstream.clone();
            let config = ClientConfig {
                room_id,
                user_id,
                shutdown,
                downstream,
            };

            tokio::spawn(start_worker(config, URL))
        };

        self.shutdown.insert(room_id, shutdown);
        self.tasks.insert(room_id, task);
        Ok(())
    }

    /// Disconnect from the specified room
    pub async fn disconnect(&mut self, room_id: i64) {
        if let Some(shutdown) = self.shutdown.remove(&room_id) {
            shutdown.store(true, Ordering::Relaxed);
            let task = self.tasks.remove(&room_id);
            if let Some(task) = task {
                let (res,) = tokio::join!(task);
                if let Err(err) = res {
                    error!("{}", err);
                }
            }
        }
    }

    /// Shutdown this client, disconnecting from all rooms
    pub fn shutdown(&mut self) {
        let ids = std::mem::take(&mut self.shutdown);
        let tasks = std::mem::take(&mut self.tasks);
        for shutdown in ids.into_values() {
            shutdown.store(true, Ordering::Relaxed);
        }
        tokio::spawn(async move {
            for task in tasks.into_values() {
                let (res,) = tokio::join!(task);
                if let Err(err) = res {
                    error!("{}", err);
                }
            }
        });
    }
}

impl Drop for BiliClient {
    fn drop(&mut self) {
        // make sure we clean up before dropping
        self.shutdown();
    }
}

/// Configuration that describes a websocket connection,
/// it is used to start a background connecting task
#[derive(Debug, Clone)]
struct ClientConfig {
    room_id: i64,
    user_id: Option<u64>,
    // shared with the top-level Client Handle & will be modified
    // by the handle to signal termination
    shutdown: Arc<AtomicBool>,
    downstream: Consumer,
}

/// Takes care of keeping the websocket connection alive in the background
/// When not shut down, this function runs in an infinite read loop. If connection is broken
/// by accident, it reconnects automatically
///
/// *`config`: Configuration of the connection
/// * `url`: Socket Server's url
///
// todo: find a way to make shutdown faster, possibly with [stream-cancel](https://github.com/jonhoo/stream-cancel)
async fn start_worker(config: ClientConfig, url: &'static str) {
    loop {
        let ClientConfig {
            room_id,
            user_id,
            shutdown,
            downstream,
        } = config.clone();
        let (cli, _) = connect_async(url).await.unwrap();
        let (mut write, read) = cli.split();

        // this task handles the sending message stream to the Bilibili's live server
        let heartbeat_task = {
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                let mut heartbeat_stream =
                    create_heartbeat_stream(room_id, user_id, shutdown).await;
                write.send_all(&mut heartbeat_stream).await.unwrap();
                // our sending stream has ended, send a close frame just for courtesy
                write.send(Message::Close(None)).await.unwrap();
            })
        };

        // this task handles reading message from Bilibili's live server and
        // send the message to downstream
        read.for_each(|msg| async {
            let msg = msg.unwrap();
            match msg {
                Message::Binary(buf) => {
                    let msg = BiliWebsocketMessage::from_binary(buf).unwrap();
                    for inner in msg.parse() {
                        let bili_msg = BiliMessage::from_raw_wesocket_message(inner);
                        if let Some(msg) = bili_msg {
                            // info!("Received Msg: {:?}", msg);
                            // send message to downstream
                            {
                                let res = downstream.send(msg);
                                if let Err(err) = res {
                                    error!("{}", err);
                                }
                            }
                        }
                    }
                }
                msg => {
                    // we don't expect Bilibili to send other types of message
                    // trace for debugging use
                    trace!("Room {} Received Message: {}", room_id, msg);
                }
            }
        })
        .await;

        // In normal execution the read task runs forever. If it is terminated either server stopped the connection
        // or we have terminated. Either case abort the write task accordingly.
        heartbeat_task.abort();

        // terminated, exit
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        warn!("Connection Lost, Reconnecting...");
    }

    // sanity check
    assert!(config.shutdown.load(Ordering::Relaxed));
    info!("Websocket Connection to Room {} Terminated", config.room_id);
}

/// Creates a message stream to Bilibili's server with the following structure:
/// [Entry Security Message]
/// | every 20s
/// V
/// Heartbeat
/// ...
async fn create_heartbeat_stream(
    room_id: i64,
    uid: Option<u64>,
    shutdown: Arc<AtomicBool>,
) -> impl Stream<Item = std::result::Result<Message, Error>> {
    let (mut tx, rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(async move {
        let entry = BiliWebsocketMessage::entry(room_id, uid);
        tx.send(entry).await.unwrap();
        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            tokio::time::sleep(Duration::from_secs(20)).await;
            tx.send(BiliWebsocketMessage::heartbeat()).await.unwrap();
        }
    });
    rx.map(|msg| Ok(Message::Binary(msg.to_vec())))
}
