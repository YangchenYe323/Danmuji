#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate lazy_static;

mod apis;
mod client;
mod config;
mod error;
mod response;
mod sender;
mod util;

use client::{BiliClient, BiliMessage};
pub(crate) use config::{RoomConfig, UserConfig};
use error::DanmujiError;
use hyper::Method;
use hyper::StatusCode;
use response::DanmujiApiResponse;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeFile,
};
use tracing::{info, warn, Level};

use axum::{
    extract::Extension,
    response::IntoResponse,
    routing::{get, get_service, post},
    Router,
};
use axum_extra::routing::SpaRouter;

use apis::user::{getLoginStatus, getQrCode, loginCheck, logout};
use sender::DanmujiSender;

use apis::room::{disconnect, getRoomStatus, roomInit};
use apis::ws::handler;
use util::*;

pub type DanmujiResult<T> = std::result::Result<T, DanmujiError>;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36";

/// The State of the application
/// Basic Architecture:
/// [BiliClient] (Receives Bilibili's Message and Sync with frontend)
/// |
/// |  tx: broadcast channel
/// V
/// Axum's Websocket Server (Subscribes [BiliClient] and relays the message to frontend)
///
/// Danmu Processing Plugins (Gift Thanks, Subscription Thanks, etc.)
/// |
/// |  sender_tx: mpsc channel
/// V
/// [DanmujiSender] (Consumes the danmu produced by plugins and posts them to Bilibili)
pub struct DanmujiState {
    // client that receives massage from Bilibili
    cli: BiliClient,
    // client that sends Bullet Screen Comments
    sender: DanmujiSender,
    // broadcast sender of bili client
    tx: broadcast::Sender<BiliMessage>,
    // sender for danmu to post
    sender_tx: tokio::sync::mpsc::UnboundedSender<String>,
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

    // setup broadcast channel & client
    let (tx, _rx) = broadcast::channel(100);
    let mut cli = BiliClient::new(tx.clone());
    // try to recover saved config
    let user = load_user_config();
    let room = load_room_config();
    info!("User: {:?}", user);
    info!("Room: {:?}", room);
    // start connection if room config is set
    if let Some(room) = &room {
        cli.start(room.room_init.room_id, user.as_ref().map(|u| u.user.uid))
            .unwrap();
    }

    // set up danmu sender
    let (sender_tx, sender_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let danmu_sender = DanmujiSender::start(sender_rx);
    if let Some(user) = user.as_ref() {
        danmu_sender.login_user(user.clone()).await.unwrap();
    }
    if let Some(room) = room.as_ref() {
        danmu_sender.connect_room(room.clone()).await.unwrap();
    }

    // todo: set up plugin executor

    // test producer
    // let tx_test = tx.clone();
    // tokio::spawn(async move {
    //     loop {
    //         // tx_test.send(BiliMessage::Danmu(DanmuMessage::default_message())).unwrap();
    //         tx_test.send(BiliMessage::Gift(GiftMessage::default_message())).unwrap();
    //         sleep(Duration::from_millis(500)).await;
    //     }
    // });

    // test danmu producer
    // let sender_tx_test = sender_tx.clone();
    // tokio::spawn(async move {
    //     loop {
    //         sender_tx_test.send("1".to_string()).unwrap();
    //         sleep(Duration::from_millis(10000)).await;
    //     }
    // });

    // initialize state
    let state = DanmujiState {
        cli,
        sender: danmu_sender,
        tx,
        sender_tx,
        user,
        room,
    };

    // single page routers
    let spa = SpaRouter::new("/assets", "frontend/dist/assets");

    let app = Router::new()
        .merge(spa) // assets
        .route("/api/loginStatus", get(getLoginStatus)) // apis
        .route("/api/qrcode", get(getQrCode))
        .route("/api/loginCheck", post(loginCheck))
        .route("/api/logout", get(logout))
        .route("/api/ws", get(handler))
        .route("/api/roomStatus", get(getRoomStatus))
        .route("/api/roomInit/:room_id", get(roomInit))
        .route("/api/disconnect", get(disconnect))
        .fallback(
            get_service(ServeFile::new("frontend/dist/index.html")).handle_error(handle_error), // serve index page as fallback
        )
        .layer(Extension(Arc::new(Mutex::new(state))));

    axum::Server::bind(&"0.0.0.0:9000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
