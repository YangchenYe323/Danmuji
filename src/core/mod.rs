//! This module contains the core functionality of a Danmuji:
//! A DanmujiCore Struct that drives a websocket client to the specified
//! live room and hold another websocket server. It streams the message received
//! from the live room to the end user through its websocket server.
mod client;
mod danmuji_core;
pub mod message;
use crate::Result;

/// users (in our case the HTTP Server) interacts with
/// Danmuji's Core by a Command channel, which can have
/// the following five types of operations
pub enum Command {
    // login using the given user cookie
    UserLogin(String),
    // unlog if we're currently logged in
    UserUnLog,
    // connect to the given live room
    ConnectRoom(u64),
    // disconnect if we're currently connected
    Disconnect,
    // change configuration
    ConfigChange,
}

#[rocket::async_trait]
pub trait DanmujiCore {
    async fn user_login(&self, raw_cookie: String) -> Result<()>;
}
