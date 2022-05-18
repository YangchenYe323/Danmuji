//! This crate provides the core functionalities of a Bilibili's
//! Websocket Client that is a wrapper around [websocket](https://docs.rs/websocket/latest/websocket/),
//! that encapsulates bilibili's network protocol, and exposes a unified publi API

mod common;
mod biliclient;

pub use biliclient::BiliClient;