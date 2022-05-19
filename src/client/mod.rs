//! This crate provides the core functionalities of a Bilibili's
//! Websocket Client that is a wrapper around [rust-websocket](https://docs.rs/websocket/0.26.4/websocket/),

mod biliclient;
mod common;
mod message;

pub use biliclient::BiliClient;
pub use common::{
	BiliMessage,
	DanmuMessage,
	GuardType,
	Medal,
};

pub(crate) use self::message::{
	BiliWebsocketInner,
	BiliWebsocketMessageBody,
	NotificationBody,
};