//! Module for User login/logout APIs

use std::{collections::HashSet, sync::Arc};

use crate::{
    config::{User, UserConfig},
    util::{delete_user_config, save_user_config},
    DanmujiApiResponse, DanmujiError, DanmujiResult, DanmujiState, USER_AGENT,
};
use axum::{Extension, extract::Json};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::warn;
use ts_rs::TS;

/// QrCode Url For Login
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/QrCode.ts")]
pub struct QrCode {
    url: String,
    oauthKey: String,
}

/// Request Path: <host>/api/qrcode
/// Request Method: GET
///
/// This interface is used for requesting a qrcode url for login
///
/// # Error:
/// Propogate HTTP Errors at the Bilibili's API
///
/// # Failure:
/// This request would fail if a user is currently logged in
///
/// # Success:
/// On success, return [QrCode] in Json Format
///
#[debug_handler]
pub async fn getQrCode() -> DanmujiResult<DanmujiApiResponse<QrCode>> {
    let cli = reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?;
    let res = cli
        .get("https://passport.bilibili.com/qrcode/getLoginUrl")
        .send()
        .await?;

    // QrCode api response
    // pub struct QrCodeResponse {
    //     code: u64,
    //     status: bool,
    //     ts: u64,
    //     data: QrCode,
    // }
    let mut res: Value = res.json().await?;
    let data = res
        .get_mut("data")
        .ok_or(DanmujiError::APIFormatError)?
        .take();

    Ok(DanmujiApiResponse::success(Some(serde_json::from_value(
        data,
    )?)))
}

/// Request Path: <host>/api/loginCheck
/// Request Method: Post
/// Reqeust Body: [QrCode]
///
/// This interface is requested for qrcode login check.
/// This is intended to be requested after the frontend has got and displayed
/// a qrcode image and the user has scanned it with there Bilibili App.
///
// # Error:
/// Propogate HTTP Errors at the Bilibili's API
///
/// # Failure:
/// This request would fail if the login has not gone through
///
/// # Success:
/// This call succeeds when the login has succeeded at Bilibili's end.
/// On success, return [User] in Json Format
/// It also succeeds when a user is already logged in.
///
#[debug_handler]
pub async fn loginCheck(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
    Json(qrcode): Json<QrCode>,
) -> DanmujiResult<DanmujiApiResponse<User>> {
    let mut state = state.lock().await;
    if let Some(user_config) = &state.user {
        // already logged in
        return Ok(DanmujiApiResponse::success(Some(user_config.user.clone())));
    }

    // extract oauthKey
    let QrCode { url: _, oauthKey } = qrcode;

    // Request API: "https://passport.bilibili.com/qrcode/getLoginInfo"
    // Request Method: Post
    // Request Form: [(oauthKey), (gourl)]
    // polls Bilibili's qrcode login API to see if login as succeeded
    // if succeeded, we retrieve the Set-Cookie header in the response
    // and fetch user configuration
    let form = vec![
        ("oauthKey", oauthKey),
        ("gourl", "https://www.bilibili.com/".to_string()),
    ];

    let cli = reqwest::Client::new();
    let res = cli
        .post("https://passport.bilibili.com/qrcode/getLoginInfo")
        .header("user-agent", USER_AGENT)
        .header("referer", "https://passport.bilibili.com/login")
        .form(&form)
        .send()
        .await?;

    // Login Check Response
    // struct LoginResponse {
    // code: u64,
    // status: bool,
    // }
    let headers = res.headers().clone();
    let login_res: Value = res.json().await?;
    let status = login_res
        .get("status")
        .ok_or(DanmujiError::APIFormatError)?
        .as_bool()
        .ok_or(DanmujiError::APIFormatError)?;

    if status {
        // the response might have multiple Set-Cookie headers
        // extract and process all of them
        let cookie = headers.get_all("Set-Cookie");
        let mut cookie_set = HashSet::new();
        for c in cookie {
            // here we know that Bilibili's cookie header doesn't contain
            // opaque bytes, hence the unwrap is justified
            let cookie_terms = c.to_str().unwrap().split(';').map(str::to_string);
            for term in cookie_terms {
                cookie_set.insert(term);
            }
        }
        let cookies: Vec<String> = cookie_set.into_iter().collect();
        let cookie_str = cookies.join(";");

        let config = UserConfig::fetch(cookie_str).await?;
        println!("User Config: {:?}", config);

        if let Err(err) = save_user_config(&config) {
            warn!("Error Saving User Config: {}", err);
        }

        // update user state and sender state
        state.sender.login_user(config.clone()).await?;
        state.user = Some(config.clone());

        return Ok(DanmujiApiResponse::success(Some(config.user)));
    }

    // login has not gone through, return failure so that
    // client can retry
    Ok(DanmujiApiResponse::failure(None))
}

/// Request Path: <host>/api/loginStatus
/// Request Method: GET
///
/// Query the login status of the server
#[debug_handler]
pub async fn getLoginStatus(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<User>> {
    let state = state.lock().await;

    let user_config = &state.user;

    if user_config.is_some() {
        // logged in
        Ok(DanmujiApiResponse::success(
            user_config.as_ref().map(|config| config.user.clone()),
        ))
    } else {
        // not logged in
        Ok(DanmujiApiResponse::failure(None))
    }
}

/// Request Path: <host>/api/logout
/// Request Method: POST
///
/// Logout the user and stop room connection if any
#[debug_handler]
pub async fn logout(
    Extension(state): Extension<Arc<Mutex<DanmujiState>>>,
) -> DanmujiResult<DanmujiApiResponse<String>> {
    let mut state = state.lock().await;

    // have not logged in
    if state.user.is_some() {
        state.user.take();
        state.sender.unlog_user().await;
    }

    // delete config file
    if let Err(err) = delete_user_config() {
        warn!("Error deleting User Config: {}", err);
    }

    Ok(DanmujiApiResponse::success(Some("".to_string())))
}
