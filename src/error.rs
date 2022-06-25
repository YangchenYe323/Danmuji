use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use serde::ser::SerializeStruct;
use serde::Serialize;
use thiserror::Error;

/// Errors of the Danmuji Application
/// todo: for some of the variants, it is not sure they're worth capturing, i.e., the application
/// can still be functioning after they occurs.
#[derive(Error, Debug)]
pub enum DanmujiError {
    /// Http request error, e.g., fail to
    /// fetch UserInfo or login url from Bilibili's API
    #[error("HTTP Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Invalid Header Value
    #[error("Invalid Header Value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    /// We need to parse SESSDATA, DedeUserID, bili_jct, etc fields from
    /// cookie string. The error represents something is missing.
    #[error("{0}")]
    CookieParse(&'static str),

    /// Forward IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    /// Forward Json Parsing Error
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),

    /// Missing expected field in Bilibili's API response
    #[error("Unexpected Bilibili's API Format, Please File an Issue")]
    APIFormatError,
}

impl DanmujiError {
    // create cookie error
    pub fn cookie(msg: &'static str) -> DanmujiError {
        DanmujiError::CookieParse(msg)
    }
}

// Serialize DanmujiError type to
// struct {
//    msg: String
// }
impl Serialize for DanmujiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DanmujiError", 1)?;
        state.serialize_field("msg", &self.to_string())?;
        state.end()
    }
}

impl IntoResponse for DanmujiError {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self).unwrap();

        // its often easiest to implement `IntoResponse` by calling other implementations
        // todo: produce more meaningful status code based on error kind
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
