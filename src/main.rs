#![allow(non_snake_case)]
#![warn(rust_2018_idioms)]

mod config;
mod monitor;
mod cors;
mod error;

use cors::CORS;
pub(crate) use config::{UserConfig, User};
use rocket::{State, Build};
use error::DanmujiError;

pub type Result<T> = core::result::Result<T, DanmujiError>;

#[macro_use] 
extern crate rocket;

use std::collections::HashSet;
use std::sync::Arc;

use rocket::serde::{
    Serialize,
    Deserialize,
    json::Json,
};
use rocket::tokio::sync::Mutex;
use reqwest::header;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct QrCode {
    url: String,
    oauthKey: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct QrCodeResponse {
    code: u64,
    status: bool,
    ts: u64,
    data: QrCode,
}

/// get qrcode url for login
#[get("/qrcode")]
async fn getQrCode() -> Result<Json<QrCode>> {
    let cli = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36")
        .build()?;
    let res = cli.get("https://passport.bilibili.com/qrcode/getLoginUrl")
        .send().await?;
    let res: QrCodeResponse = res.json().await?;
    Ok(Json(res.data))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct LoginResponse {
    code: u64,
    status: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserInfoResponse {
    code: String,
    msg: String,
    message: String,
    data: User,
}

#[post("/loginCheck", data = "<login_data>")]
async fn loginCheck(login_data: Json<QrCode>, state: &State<DanmujiState>) -> Result<String> {
    let QrCode { url: _, oauthKey } = login_data.into_inner();
    let mut headers = header::HeaderMap::new();
    headers.insert("referer", header::HeaderValue::from_static("https://passport.bilibili.com/login"));
    headers.insert("user-agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36"));

    let mut form = vec![];
    form.push(("oauthKey", oauthKey));
    form.push(("gourl", "https://www.bilibili.com/".to_string()));
    
    let cli = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;
        
    
    let res = cli.post("https://passport.bilibili.com/qrcode/getLoginInfo")
        .form(&form)
        .send()
        .await?;
    
    let headers = res.headers().clone();    
    let login_res: LoginResponse = res.json().await?;

    println!("{:?}", login_res);

    if login_res.status {
        // update user config
        let cookie = headers.get_all("Set-Cookie");
        
        let mut cookie_set = HashSet::new();

        for c in cookie {
            let cookie_terms = c.to_str()?.split(";").map(str::to_string);
            for term in cookie_terms {
                cookie_set.insert(term);
            }
        }

        let cookies: Vec<String> = cookie_set.into_iter().collect();

        let cookie_str = cookies.join(";");

        println!("{}", cookie_str);

        // get user info
        let mut headers = header::HeaderMap::new();
        headers.insert("user-agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36"));
        headers.insert("cookie", header::HeaderValue::from_str(&cookie_str)?);
        let cli = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;
        let res = cli.get("https://api.live.bilibili.com/User/getUserInfo")
            .send()
            .await?;
        
        let user: UserInfoResponse = res.json().await?;
        println!("{:?}", user);

        let config = UserConfig {
            cookie: cookie_str,
            user: Some(user.data),
        };

        // update user state
        let mut state = state.config.lock().await;
        *state = Some(config);

        return Ok(String::from("Success"));
    }

    Ok(String::from("failed"))
}

#[post("/logout")]
async fn logout(state: &State<DanmujiState>) -> Result<String> {
    let mut config = state.config.lock().await;
    config.take();
    Ok("".to_string())
}

struct DanmujiState {
    config: Arc<Mutex<Option<UserConfig>>>,
}

fn rocket(state: DanmujiState) -> rocket::Rocket<Build> {
    rocket::build()
        .attach(CORS)
        .manage(state)
        .mount("/", routes![index, getQrCode, loginCheck, logout])
}

#[rocket::main]
async fn main() {
    let state = DanmujiState {
        config: Arc::new(Mutex::new(None)),
    };

    let server = rocket(state);

    let _result = server.launch().await;

    // this is reachable only after `Shutdown::notify()` or `Ctrl+C`.
    println!("Rocket: deorbit.");
}