#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![warn(rust_2018_idioms)]

mod client;
mod config;
mod error;
mod response;

use client::{BiliClient, BiliMessage};
pub(crate) use config::{Room, RoomConfig, User, UserConfig};
use error::DanmujiError;
use futures::{SinkExt, StreamExt};
use hyper::Method;
use response::DanmujiApiResponse;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant, Sleep};
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, error, info, warn, Level};
use ts_rs::TS;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Extension, Json, Path},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::client::{DanmuMessage, GiftMessage, GuardType};

pub type DanmujiResult<T> = std::result::Result<T, DanmujiError>;

pub const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36";

/// QrCode Url For Login
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/QrCode.ts")]
pub struct QrCode {
    url: String,
    oauthKey: String,
}

fn index() -> &'static str {
    "Hello, world!"
}

/// Request Path: <host>/qrcode
/// Request Method: GET
///
/// This interface is used for requesting a qrcode url for login
///
/// # Error:
/// Propogate HTTP Errors at the Bilibili's API
///
/// # Failure:
/// This request would fail if a user is currently logged in
///
/// # Success:
/// On success, return [QrCode] in Json Format
///
async fn getQrCode() -> DanmujiResult<DanmujiApiResponse<QrCode>> {
    let cli = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36")
        .build()?;
    let res = cli
        .get("https://passport.bilibili.com/qrcode/getLoginUrl")
        .send()
        .await?;

    // QrCode api response
    // pub struct QrCodeResponse {
    //     code: u64,
    //     status: bool,
    //     ts: u64,
    //     data: QrCode,
    // }
    let mut res: Value = res.json().await?;
    let data = res
        .get_mut("data")
        .ok_or(DanmujiError::APIFormatError)?
        .take();

    Ok(DanmujiApiResponse::success(Some(serde_json::from_value(
        data,
    )?)))
}

/// Request Path: <host>/loginCheck
/// Request Method: Post
/// Reqeust Body: [QrCode]
///
/// This interface is requested for qrcode login check.
/// This is intended to be requested after the frontend has got and displayed
/// a qrcode image and the user has scanned it with there Bilibili App.
///
// # Error:
/// Propogate HTTP Errors at the Bilibili's API
///
/// # Failure:
/// This request would fail if the login has not gone through
///
/// # Success:
/// This call succeeds when the login has succeeded at Bilibili's end.
/// On success, return [User] in Json Format
/// It also succeeds when a user is already logged in.
///
async fn loginCheck(
    Json(qrcode): Json<QrCode>,
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<User>> {
    let mut state = state.lock().await;
    if let Some(user_config) = &state.user {
        // already logged in
        return Ok(DanmujiApiResponse::success(Some(user_config.user.clone())));
    }

    // extract oauthKey
    let QrCode { url: _, oauthKey } = qrcode;

    // Request API: "https://passport.bilibili.com/qrcode/getLoginInfo"
    // Request Method: Post
    // Request Form: [(oauthKey), (gourl)]
    // polls Bilibili's qrcode login API to see if login as succeeded
    // if succeeded, we retrieve the Set-Cookie header in the response
    // and fetch user configuration
    let mut form = vec![];
    form.push(("oauthKey", oauthKey));
    form.push(("gourl", "https://www.bilibili.com/".to_string()));

    let cli = reqwest::Client::new();
    let res = cli
        .post("https://passport.bilibili.com/qrcode/getLoginInfo")
        .header("user-agent", USER_AGENT)
        .header("referer", "https://passport.bilibili.com/login")
        .form(&form)
        .send()
        .await?;

    // Login Check Response
    // struct LoginResponse {
    // code: u64,
    // status: bool,
    // }
    let headers = res.headers().clone();
    let login_res: Value = res.json().await?;
    let status = login_res
        .get("status")
        .ok_or(DanmujiError::APIFormatError)?
        .as_bool()
        .ok_or(DanmujiError::APIFormatError)?;

    if status {
        // the response might have multiple Set-Cookie headers
        // extract and process all of them
        let cookie = headers.get_all("Set-Cookie");
        let mut cookie_set = HashSet::new();
        for c in cookie {
            let cookie_terms = c.to_str()?.split(";").map(str::to_string);
            for term in cookie_terms {
                cookie_set.insert(term);
            }
        }
        let cookies: Vec<String> = cookie_set.into_iter().collect();
        let cookie_str = cookies.join(";");

        let config = UserConfig::fetch(cookie_str).await?;
        println!("User Config: {:?}", config);

        if let Err(err) = save_user_config(&config) {
            warn!("Error Saving User Config: {}", err);
        }

        // update user state
        let return_user = config.user.clone();
        state.user = Some(config);

        return Ok(DanmujiApiResponse::success(Some(return_user)));
    }

    // login has not gone through, return failure so that
    // client can retry
    Ok(DanmujiApiResponse::failure(None))
}

/// Request Path: <host>/loginStatus
/// Request Method: GET
///
/// Query the login status of the server
async fn getLoginStatus(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<User>> {
    let state = state.lock().await;

    let user_config = &state.user;

    if user_config.is_some() {
        // logged in
        Ok(DanmujiApiResponse::success(
            user_config.as_ref().map(|config| config.user.clone()),
        ))
    } else {
        // not logged in
        Ok(DanmujiApiResponse::failure(None))
    }
}

/// Request Path: <host>/logout
/// Request Method: GET
///
/// Logout the user and stop room connection if any
async fn logout(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<String>> {
    let mut state = state.lock().await;

    // have not logged in
    if state.user.is_some() {
        state.user.take();
    }

    Ok(DanmujiApiResponse::success(Some("".to_string())))
}

/// Request Path: <host>/roomStatus
/// Request Method: GET
///
/// Query which room this server is connected to
async fn getRoomStatus(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<Room>> {
    let state = state.lock().await;

    let room_config = &state.room;

    if room_config.is_some() {
        Ok(DanmujiApiResponse::success(
            room_config.as_ref().map(|config| config.room.clone()),
        ))
    } else {
        Ok(DanmujiApiResponse::failure(None))
    }
}

/// Request Path: <host>/disconnect
/// Request Method: GET
///
/// Disconnect from current room.
/// Always succeed
async fn disconnect(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<()>> {
    let mut state = state.lock().await;

    state.cli.shutdown();
    state.room.take();

    // delete config file
    if let Err(err) = std::fs::remove_file("room.json") {
        warn!("Fail deleting room config: {}", err);
    }

    Ok(DanmujiApiResponse::success(None))
}

/// Request Path: <host>/roomInit/:room_id
/// Request Method: GET
///
/// try to set up a websocket connection to the live room of specified
/// id.
///
///
/// # Error:
/// Propogate HTTP error occured at Bilibili's API
///
/// # Failure:
/// Fails if (a). the given room_id is not a valid room
/// or (b). we have already connected to a room.
///
/// # Success:
/// On success, client is connected to the specified room
async fn roomInit(
    Path(room_id): Path<i64>,
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<Room>> {
    // a room is already connected to
    let mut state = state.lock().await;

    // already connected
    if state.room.is_some() {
        return Ok(DanmujiApiResponse::failure(None));
    }

    // fetch room config
    let room_config = RoomConfig::fetch(room_id).await?;
    if room_config.room_init.room_id == 0 {
        return Ok(DanmujiApiResponse::failure(None));
    }

    // valid room, connect
    if let Err(err) = save_room_config(&room_config) {
        warn!("Save room config failure: {}", err);
    }

    let return_room = room_config.room.clone();
    let room_id = room_config.room_init.room_id;
    state.room = Some(room_config);

    let uid = state.user.as_ref().map(|u| u.user.uid);

    // start client
    let tx = state.tx.clone();
    let cli = &mut state.cli;
    cli.shutdown();
    cli.set_downstream(Some(tx));
    cli.start(room_id, uid)?;

    // Ok
    Ok(DanmujiApiResponse::success(Some(return_room)))
}

/// The State of the application
struct DanmujiState {
    // client that receives massage from Bilibili
    cli: BiliClient,
    // broadcast sender of bili client
    tx: broadcast::Sender<BiliMessage>,
    // user configuration
    user: Option<UserConfig>,
    // room configuration
    room: Option<RoomConfig>,
}

#[tokio::main]
async fn main() {
    // set log collector
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(Level::DEBUG)
        .init();
    info!("Logger Initialized");

    // setup broadcast channel
    let (tx, _rx) = broadcast::channel(100);
    let mut cli = BiliClient::new(tx.clone());
    // try to recover saved config
    let user = load_user_config();
    let room = load_room_config();
    info!("User Config: {:?}", user);
    info!("Room Config: {:?}", room);

    // test producer
    // let tx_test = tx.clone();
    // tokio::spawn(async move {
    //     loop {
    //         // tx_test.send(BiliMessage::Danmu(DanmuMessage::default_message())).unwrap();
    //         tx_test.send(BiliMessage::Gift(GiftMessage::default_message())).unwrap();
    //         sleep(Duration::from_millis(500)).await;
    //     }
    // });

    // start connection if room config is set
    if let Some(room) = &room {
        cli.start(room.room_init.room_id, user.as_ref().map(|u| u.user.uid))
            .unwrap();
    }

    // initialize state
    let state = DanmujiState {
        cli,
        tx,
        user,
        room,
    };

    //cors
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/loginStatus", get(getLoginStatus))
        .route("/qrcode", get(getQrCode))
        .route("/loginCheck", post(loginCheck))
        .route("/logout", get(logout))
        .route("/ws", get(handler))
        .route("/roomStatus", get(getRoomStatus))
        .route("/roomInit/:room_id", get(roomInit))
        .route("/disconnect", get(disconnect))
        .layer(Extension(Arc::new(Mutex::new(state))))
        .layer(cors);

    axum::Server::bind(&"0.0.0.0:9000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn save_json(object: &impl Serialize, path: impl AsRef<std::path::Path>) -> DanmujiResult<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, object)?;
    Ok(())
}

fn load_json<T: DeserializeOwned>(path: impl AsRef<std::path::Path>) -> Option<T> {
    let file = OpenOptions::new().read(true).open(path).ok()?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).ok()
}

fn save_user_config(config: &UserConfig) -> DanmujiResult<()> {
    save_json(config, "user.json")
}

fn save_room_config(config: &RoomConfig) -> DanmujiResult<()> {
    save_json(config, "room.json")
}

fn load_user_config() -> Option<UserConfig> {
    load_json("user.json")
}

fn load_room_config() -> Option<RoomConfig> {
    load_json("room.json")
}

// heartbeat timeout in seconds
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

/// Request Path: ws://<host>/ws
///
/// Set up a websocket connection with this server, this server
/// will forward all the messages from Bilibili to the client
async fn handler(
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
