use hyper::StatusCode;
use axum::response::{
    IntoResponse,
    Response
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DanmujiError {
    /// Http request error, e.g., fail to 
    /// fetch UserInfo or login url from Bilibili's
    /// API
    #[error("HTTP Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Fail to Parse header
    #[error("Header Parsing Error: {0}")]
    HeaderParse(#[from] reqwest::header::ToStrError),
    /// Invalid Header Value
    #[error("Invalid Header Value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    /// We need to parse SESSDATA, DedeUserID, bili_jct, etc fields from
    /// cookie string. Throw the error if some field is missing
    #[error("{0}")]
    CookieParse(&'static str),
    /// Forward IO Error
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    /// Forward Json Parsing Error
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    /// Forward Websocket Error
    #[error("{0}")]
    WebsocketError(#[from] websocket::WebSocketError),
}

impl DanmujiError {
    // create cookie error
    pub fn cookie(msg: &'static str) -> DanmujiError {
        DanmujiError::CookieParse(msg)
    }
}

impl IntoResponse for DanmujiError {
    fn into_response(self) -> Response {

        let body = self.to_string();

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}