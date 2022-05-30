//! Modules for room connection/disconnection APIs

use axum::{extract::Path, Extension};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    config::{Room, RoomConfig},
    util::{delete_room_config, save_room_config},
    DanmujiApiResponse, DanmujiResult, DanmujiState,
};

/// Request Path: <host>/api/roomStatus
/// Request Method: GET
///
/// Query which room this server is connected to
pub async fn getRoomStatus(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<Room>> {
    let state = state.lock().await;

    let room_config = &state.room;

    if room_config.is_some() {
        Ok(DanmujiApiResponse::success(
            room_config.as_ref().map(|config| config.room.clone()),
        ))
    } else {
        Ok(DanmujiApiResponse::failure(None))
    }
}

/// Request Path: <host>/api/disconnect
/// Request Method: GET
///
/// Disconnect from current room.
/// Always succeed
pub async fn disconnect(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<()>> {
    let mut state = state.lock().await;

    state.cli.shutdown();
    if state.room.is_some() {
        state.room.take();
        state.sender.disconnect_room().await;
    }

    // delete config file
    if let Err(err) = delete_room_config() {
        warn!("Fail deleting room config: {}", err);
    }

    Ok(DanmujiApiResponse::success(None))
}

/// Request Path: <host>/api/roomInit/:room_id
/// Request Method: GET
///
/// try to set up a websocket connection to the live room of specified
/// id.
///
///
/// # Error:
/// Propogate HTTP error occured at Bilibili's API
///
/// # Failure:
/// Fails if (a). the given room_id is not a valid room
/// or (b). we have already connected to a room.
///
/// # Success:
/// On success, client is connected to the specified room
pub async fn roomInit(
    Path(room_id): Path<i64>,
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<Room>> {
    // a room is already connected to
    let mut state = state.lock().await;

    // already connected
    if state.room.is_some() {
        return Ok(DanmujiApiResponse::failure(None));
    }

    // fetch room config
    let room_config = RoomConfig::fetch(room_id).await?;
    if room_config.room_init.room_id == 0 {
        // invalid room id
        return Ok(DanmujiApiResponse::failure(None));
    }

    // valid room, connect
    if let Err(err) = save_room_config(&room_config) {
        warn!("Save room config failure: {}", err);
    }

    let return_room = room_config.room.clone();
    let room_id = room_config.room_init.room_id;
    state.sender.connect_room(room_config.clone()).await?;
    state.room = Some(room_config);

    let uid = state.user.as_ref().map(|u| u.user.uid);

    // start client
    let tx = state.tx.clone();
    let cli = &mut state.cli;
    cli.shutdown();
    cli.set_downstream(Some(tx));
    cli.start(room_id, uid)?;

    // Ok
    Ok(DanmujiApiResponse::success(Some(return_room)))
}
