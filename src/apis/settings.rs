//! This module contains Danmuji's Web API for changing settings,
//! e.g., auto gift thanks.
use axum::{Extension, Json};
use axum_macros::debug_handler;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    plugins::GiftThankConfig, util::save_thank_config, DanmujiApiResponse, DanmujiResult,
    DanmujiState,
};

/// Request Path: <host>/api/getGiftConfig
/// Request Method: GET
///
/// Query the current Gift Config
pub async fn queryGiftConfig(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<GiftThankConfig>> {
    let state = state.lock().await;
    let config = state.thanker.get_config().await;
    Ok(DanmujiApiResponse::success(config))
}

/// Request Path <host>/api/setGiftConfig
/// Request Method: POST
/// Request Body: Json<GiftThankConfig>
///
/// set server's gift thank config
#[debug_handler]
pub async fn setGiftConfig(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
    Json(config): Json<GiftThankConfig>,
) -> DanmujiResult<DanmujiApiResponse<()>> {
    if let Err(err) = save_thank_config(&config) {
        warn!("Fail Saving Thank Config: {}", err);
    }
    let state = state.lock().await;
    state.thanker.set_config(config).await;
    Ok(DanmujiApiResponse::success(None))
}
