use thiserror::Error;

#[derive(Error, Debug, Responder)]
pub enum DanmujiError {
	#[error("Reqwest Error")]
	Reqwest(String),

	#[error("Error Parsing Header")]
	HeaderParse(String),

	#[error("Invalid Header Value")]
	InvalidHeaderValue(String),
}

impl From<reqwest::Error> for DanmujiError {
	fn from(_err: reqwest::Error) -> DanmujiError {
		DanmujiError::Reqwest("Reqwest Error".to_string())
	}
}

impl From<reqwest::header::ToStrError> for DanmujiError {
	fn from(_err: reqwest::header::ToStrError) -> DanmujiError {
		DanmujiError::HeaderParse("Header Parse Error".to_string())
	}
}

impl From<reqwest::header::InvalidHeaderValue> for DanmujiError {
	fn from(_err: reqwest::header::InvalidHeaderValue) -> DanmujiError {
		DanmujiError::HeaderParse("Header Parse Error".to_string())
	}
}