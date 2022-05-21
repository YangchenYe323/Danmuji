use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// API response of Danmuji Server
#[derive(Serialize, Debug)]
pub struct DanmujiApiResponse<T: Serialize> {
    success: bool,
    payload: Option<T>,
}

impl<T: Serialize> DanmujiApiResponse<T> {
    pub fn success(payload: Option<T>) -> Self {
        Self {
            success: true,
            payload,
        }
    }

    pub fn failure(payload: Option<T>) -> Self {
        Self {
            success: false,
            payload,
        }
    }
}

impl<T: Serialize> IntoResponse for DanmujiApiResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
