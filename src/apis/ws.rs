use crate::DanmujiState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

// heartbeat timeout in seconds
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

/// Request Path: ws://<host>/ws
///
/// Set up a websocket connection with this server, this server
/// will forward all the messages from Bilibili to the client
pub async fn handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> impl IntoResponse {
    info!("Get Websocket Upgrade Request");
    ws.on_upgrade(|ws| handle_socket(ws, state))
}

/// Handles a websocket connection
async fn handle_socket(socket: WebSocket, state: Arc<Mutex<DanmujiState>>) {
    info!("Weosocket Connection Established");
    let (mut sender, receiver) = socket.split();

    // state.tx is the upstream producer of all the bilibili messages
    // received from [BiliClient]
    let mut rx = state.lock().await.tx.subscribe();

    // This task will receive incoming BiliMessages and forward to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // This task will monitor user heartbeat, and abort connection
    // if we don't receive heartbeat in timeout
    let mut heartbeat_task = tokio::spawn(async move {
        // move receiver into the future
        let mut socket_receiver = receiver;

        // this task monitors timer
        let sleep = tokio::time::sleep(HEARTBEAT_TIMEOUT);
        tokio::pin!(sleep);

        loop {
            let mut recv_task = tokio::spawn(async move {
                if let Some(Ok(msg)) = socket_receiver.next().await {
                    debug!("Msg from websocket client: {:?}", msg);
                    return Some(socket_receiver);
                }
                // todo: process other kinds of user messages and errors (Close frames, etc.)
                None
            });

            tokio::select! {
                _ = (&mut sleep) => {
                    // timeout fired without heartbeat
                    // abort connection
                    warn!("Heartbeat is not collected in time");
                    recv_task.abort();
                    break;
                }
                returned_receiver = (&mut recv_task) => {
                    match returned_receiver {
                        // received heartbeat
                        Ok(Some(recv)) => {
                            // reset receiver for next loop
                            socket_receiver = recv;
                            // reset timeout
                            sleep.as_mut().reset(
                                Instant::now() + HEARTBEAT_TIMEOUT
                            )
                        }

                        // todo: is there a better return value?
                        Ok(None) => {
                            break;
                        }

                        Err(err) => {
                            error!("{}", err);
                            break;
                        }
                    }
                }
            };
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => heartbeat_task.abort(),
        _ = (&mut heartbeat_task) => send_task.abort(),
    };

    info!("Websocket Diconnected")
}
