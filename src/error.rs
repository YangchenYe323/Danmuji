use thiserror::Error;

#[derive(Error, Debug, Responder)]
pub enum DanmujiError {
    #[error("Reqwest Error")]
    Reqwest(String),

    #[error("Error Parsing Header")]
    HeaderParse(String),

    #[error("Invalid Header Value")]
    InvalidHeaderValue(String),

    #[error("Invalid Cookie String")]
    CookieParse(String),

    #[error("IO Error")]
    IoError(String),

    #[error("Json Serialization Error")]
    JsonError(String),
}

impl DanmujiError {
    pub fn cookie(msg: &str) -> DanmujiError {
        DanmujiError::CookieParse(msg.to_string())
    }
}

impl From<reqwest::Error> for DanmujiError {
    fn from(err: reqwest::Error) -> DanmujiError {
        DanmujiError::Reqwest(err.to_string())
    }
}

impl From<reqwest::header::ToStrError> for DanmujiError {
    fn from(err: reqwest::header::ToStrError) -> DanmujiError {
        DanmujiError::HeaderParse(err.to_string())
    }
}

impl From<reqwest::header::InvalidHeaderValue> for DanmujiError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> DanmujiError {
        DanmujiError::HeaderParse(err.to_string())
    }
}

impl From<std::io::Error> for DanmujiError {
    fn from(err: std::io::Error) -> DanmujiError {
        DanmujiError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for DanmujiError {
    fn from(err: serde_json::Error) -> DanmujiError {
        DanmujiError::JsonError(err.to_string())
    }
}
