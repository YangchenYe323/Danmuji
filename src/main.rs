//! The main binary of Danmuji,
//! It provides websocket client service to a Bilibili live room
//! and supports online configuration change via a web server

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

#[macro_use]
extern crate lazy_static;

mod apis;
mod client;
mod config;
mod error;
mod plugins;
mod response;
mod sender;
mod util;

use axum::{
  extract::Extension,
  response::IntoResponse,
  routing::{get, get_service, post},
  Router,
};
use axum_extra::routing::SpaRouter;
use client::{BiliClient, BiliMessage};
pub(crate) use config::{RoomConfig, UserConfig};
use error::DanmujiError;
use hyper::StatusCode;
use response::DanmujiApiResponse;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tower_http::services::ServeFile;
use tracing::{info, warn};
use tracing_subscriber::filter::targets::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use apis::user::{getLoginStatus, getQrCode, loginCheck, logout};
use sender::DanmujiSender;

use apis::room::{disconnect, getRoomStatus, roomInit};
use apis::settings::{queryGiftConfig, setGiftConfig};
use apis::ws::handler;
use util::*;

use crate::plugins::GiftThanker;

/// Result Type used by Danmuji
pub type DanmujiResult<T> = std::result::Result<T, DanmujiError>;

/// User agent used by Danmuji
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36";

lazy_static! {
  static ref INDEX_FILE: PathBuf = PROJECT_ROOT.join("frontend/dist/index.html");
  static ref ASSETS_DIR: PathBuf = PROJECT_ROOT.join("frontend/dist/assets");
}

/// The State of the application
/// Basic Architecture:
/// [BiliClient] (Receives Bilibili's Message and Sync with frontend)
/// |
/// |  tx: broadcast channel
/// V
/// Axum's Websocket Server (Subscribes [BiliClient] and relays the message to frontend)
/// &
/// Danmu Processing Plugins (Gift Thanks, Subscription Thanks, etc.)
/// |
/// |  sender_tx: mpsc channel
/// V
/// [DanmujiSender] (Consumes the danmu produced by plugins and posts them to Bilibili)
#[derive(Debug)]
pub struct DanmujiState {
  // client that receives massage from Bilibili
  cli: BiliClient,
  // client that sends Bullet Screen Comments
  sender: DanmujiSender,
  // thank gift component
  thanker: GiftThanker,
  #[allow(dead_code)]
  // broadcast channel for subscription
  tx: broadcast::Sender<BiliMessage>,
  #[allow(dead_code)]
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
  let filter_layer =
    Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info")).unwrap();
  let format_layer = tracing_subscriber::fmt::layer();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(format_layer)
    .init();
  info!("Logger Initialized");

  // setup broadcast channel & client
  let (tx, rx) = broadcast::channel(100);
  let mut cli = BiliClient::new(tx.clone());
  // try to recover saved config
  let user = load_user_config();
  let room = load_room_config();
  info!("User: {:?}", user);
  info!("Room: {:?}", room);
  // start connection if room config is set
  if let Some(room) = &room {
    cli
      .start(room.room_init.room_id, user.as_ref().map(|u| u.user.uid))
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

  // plugin: gift thanker
  let gift_thank_config = load_thank_config();
  let thanker = GiftThanker::start(gift_thank_config, rx, sender_tx.clone());

  // initialize state
  let state = DanmujiState {
    cli,
    sender: danmu_sender,
    thanker,
    tx,
    sender_tx,
    user,
    room,
  };

  // single page routers
  let spa = SpaRouter::new("/assets", ASSETS_DIR.as_path());

  let app = Router::new()
    .merge(spa) // assets
    .route("/api/loginStatus", get(getLoginStatus)) // apis
    .route("/api/qrcode", get(getQrCode))
    .route("/api/loginCheck", post(loginCheck))
    .route("/api/logout", post(logout))
    .route("/api/ws", get(handler))
    .route("/api/roomStatus", get(getRoomStatus))
    .route("/api/roomInit/:room_id", post(roomInit))
    .route("/api/disconnect", post(disconnect))
    .route("/api/getGiftConfig", get(queryGiftConfig))
    .route("/api/setGiftConfig", post(setGiftConfig))
    .fallback_service(
      get_service(ServeFile::new(INDEX_FILE.as_path())).handle_error(handle_error), // serve index page as fallback
    )
    .layer(Extension(Arc::new(Mutex::new(state))));

  axum::Server::bind(&"0.0.0.0:9000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn handle_error(_err: impl std::error::Error) -> impl IntoResponse {
  (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
