#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![warn(rust_2018_idioms)]

mod client;
mod config;
mod cors;
mod error;

use axum::extract::ws::Message;
use client::{BiliClient, BiliMessage};
use config::BulletScreenConfig;
pub(crate) use config::{Room, RoomConfig, RoomInit, User, UserConfig, WsConfig};
use cors::CORS;
use error::DanmujiError;
use futures::{StreamExt, SinkExt};
use reqwest::header;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::sync::{
    Arc, Mutex
};
use std::time::{Duration};
use tracing::{info, Level, error, warn, debug};

pub type DanmujiResult<T> = std::result::Result<T, DanmujiError>;

pub const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36";

/// QrCode Url For Login
#[derive(Serialize, Deserialize)]
pub struct QrCode {
    url: String,
    oauthKey: String,
}

/// QrCode api response
#[derive(Serialize, Deserialize)]
pub struct QrCodeResponse {
    code: u64,
    status: bool,
    ts: u64,
    data: QrCode,
}

/// Login Check Response
#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    code: u64,
    status: bool,
}

/// UserInfo Query Response
#[derive(Debug, Serialize, Deserialize)]
struct UserInfoResponse {
    code: String,
    msg: String,
    message: String,
    data: User,
}

fn index() -> &'static str {
    "Hello, world!"
}

/// get qrcode url for login
// async fn getQrCode() -> DanmujiResult<Json<QrCode>> {
//     let cli = reqwest::ClientBuilder::new()
//         .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36")
//         .build()?;
//     let res = cli
//         .get("https://passport.bilibili.com/qrcode/getLoginUrl")
//         .send()
//         .await?;
//     let res: QrCodeResponse = res.json().await?;
//     Ok(Json(res.data))
// }

// async fn loginCheck(
//     login_data: Json<QrCode>,
//     state: &State<DanmujiState>,
// ) -> DanmujiResult<String> {
//     let QrCode { url: _, oauthKey } = login_data.into_inner();
//     let mut headers = header::HeaderMap::new();
//     headers.insert(
//         "referer",
//         header::HeaderValue::from_static("https://passport.bilibili.com/login"),
//     );
//     headers.insert("user-agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36"));

//     let mut form = vec![];
//     form.push(("oauthKey", oauthKey));
//     form.push(("gourl", "https://www.bilibili.com/".to_string()));

//     let cli = reqwest::ClientBuilder::new()
//         .default_headers(headers)
//         .build()?;

//     let res = cli
//         .post("https://passport.bilibili.com/qrcode/getLoginInfo")
//         .form(&form)
//         .send()
//         .await?;

//     let headers = res.headers().clone();
//     let login_res: LoginResponse = res.json().await?;

//     println!("{:?}", login_res);

//     if login_res.status {
//         // update user config
//         let cookie = headers.get_all("Set-Cookie");

//         let mut cookie_set = HashSet::new();

//         for c in cookie {
//             let cookie_terms = c.to_str()?.split(";").map(str::to_string);
//             for term in cookie_terms {
//                 cookie_set.insert(term);
//             }
//         }

//         let cookies: Vec<String> = cookie_set.into_iter().collect();

//         let cookie_str = cookies.join(";");

//         println!("{}", cookie_str);

//         let config = UserConfig::fetch(cookie_str).await?;
//         println!("User Config: {:?}", config);

//         save_user_config(&config)?;

//         // update user state
//         let mut state = state.config.lock().await;
//         *state = Some(config);

//         return Ok(String::from("Success"));
//     }

//     Ok(String::from("failed"))
// }

// async fn logout(state: &State<DanmujiState>) -> DanmujiResult<String> {
//     let mut config = state.config.lock().await;
//     config.take();
//     Ok("".to_string())
// }

#[derive(Debug, Serialize, Deserialize)]
struct RoomInitResponse {
    code: u8,
    msg: String,
    message: String,
    data: RoomInit,
}

#[derive(Debug, Serialize, Deserialize)]
struct RoomResponse {
    code: u8,
    msg: String,
    message: String,
    data: Room,
}

#[derive(Debug, Serialize, Deserialize)]
struct WsConfigResponse {
    code: u8,
    message: String,
    ttl: u8,
    data: WsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct BulletScreenPropertyResponse {
    code: u8,
    data: BulletScreenData,
}

#[derive(Debug, Serialize, Deserialize)]
struct BulletScreenData {
    property: BulletScreenConfig,
}

// Interface for Test Use
// Connect to the given room id using the user credential we hold
// and start the websocket client to monitor incoming messages
async fn roomInit(
    Path(room_id): Path<i64>,
    Extension(cli): Extension<Arc<Mutex<BiliClient>>>,
    Extension(config): Extension<Arc<Option<UserConfig>>>
) ->  &'static str {
        // bullet screen config
        // let bc = BulletScreenConfig::fetch(&ws, &state).await.unwrap();
        // println!("{:?}", bc);
        // let uid = state.user.uid;

    if let Some(config) = config.as_ref() {
        let uid = config.user.uid;
        {
            let mut cli = cli.lock().unwrap();
            cli.start(room_id, Some(uid)).unwrap();
        }
    }


    "Hello"
}

use axum::{
    extract::{Extension, Path},
    extract::ws::{WebSocketUpgrade, WebSocket},
    routing::get,
    response::IntoResponse,
    Router,
};
use tokio::sync::broadcast;

#[derive(Clone)]
struct DanmujiState {
    tx: broadcast::Sender<BiliMessage>,
}

#[tokio::main]
async fn main() {
    // set log collector
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(Level::DEBUG)
        .init();
    info!("Logger Initialized");

    let (tx, _rx) = broadcast::channel(100);
    let cli = BiliClient::new(tx.clone());

    let state = DanmujiState {
        tx,
    };

    let config = load_user_config();

    let app = Router::new()
        .route("/ws", get(handler))
        .route("/:room_id", get(roomInit))
        .layer(Extension(Arc::new(Mutex::new(cli)))) // state: BiliClient
        .layer(Extension(state)) // state: Synchronization States
        .layer(Extension(Arc::new(config))); // state: User Configuration

    axum::Server::bind(&"0.0.0.0:9000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

}

fn save_user_config(config: &UserConfig) -> DanmujiResult<()> {
    let options = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("config.json")
        .unwrap();

    let writer = BufWriter::new(options);

    serde_json::to_writer(writer, config).unwrap();

    Ok(())
}

fn load_user_config() -> Option<UserConfig> {
    // try to restore config from file
    let file = OpenOptions::new().read(true).open("config.json");

    if let Ok(file) = file {
        let reader = BufReader::new(file);
        // if parse failed, return None
        // otherwise return config object
        serde_json::from_reader(reader).ok()
    } else {
        // no file found, return None
        None
    }
}

// heartbeat timeout in seconds
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

/// Websocket handler
async fn handler(ws: WebSocketUpgrade, Extension(state): Extension<DanmujiState>) -> impl IntoResponse {
    info!("Get Websocket Upgrade Request");
    ws.on_upgrade(|ws| handle_socket(ws, state))
}

/// Handles a websocket connection
async fn handle_socket(socket: WebSocket, state:DanmujiState) {
    info!("Weosocket Connection Established");
    let (mut sender, receiver) = socket.split();
    
    // state.tx is the upstream producer of all the bilibili messages 
    // received from [BiliClient]
    let mut rx = state.tx.subscribe();

    // This task will receive incoming BiliMessages and forward to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.is_err() {
                break;
            }
        }
    });


    // This task will monitor user heartbeat, and abort connection
    // if we don't receive heartbeat in a given timeout
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
