#![allow(non_snake_case)]
#![warn(rust_2018_idioms)]

mod config;
mod core;
mod cors;
mod error;

#[macro_use]
extern crate rocket;

use byteorder::{BigEndian, WriteBytesExt};
use config::BulletScreenConfig;
pub(crate) use config::{Room, RoomConfig, RoomInit, User, UserConfig, WsConfig};
use cors::CORS;
use error::DanmujiError;
use reqwest::header;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::{Build, State};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;
use websocket::{ClientBuilder, Message};

pub type Result<T> = std::result::Result<T, DanmujiError>;

pub const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36";

/// QrCode Url For Login
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct QrCode {
    url: String,
    oauthKey: String,
}

/// QrCode api response
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct QrCodeResponse {
    code: u64,
    status: bool,
    ts: u64,
    data: QrCode,
}

/// Login Check Response
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct LoginResponse {
    code: u64,
    status: bool,
}

/// UserInfo Query Response
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserInfoResponse {
    code: String,
    msg: String,
    message: String,
    data: User,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

/// get qrcode url for login
#[get("/qrcode")]
async fn getQrCode() -> Result<Json<QrCode>> {
    let cli = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36")
        .build()?;
    let res = cli
        .get("https://passport.bilibili.com/qrcode/getLoginUrl")
        .send()
        .await?;
    let res: QrCodeResponse = res.json().await?;
    Ok(Json(res.data))
}

#[post("/loginCheck", data = "<login_data>")]
async fn loginCheck(login_data: Json<QrCode>, state: &State<DanmujiState>) -> Result<String> {
    let QrCode { url: _, oauthKey } = login_data.into_inner();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "referer",
        header::HeaderValue::from_static("https://passport.bilibili.com/login"),
    );
    headers.insert("user-agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36"));

    let mut form = vec![];
    form.push(("oauthKey", oauthKey));
    form.push(("gourl", "https://www.bilibili.com/".to_string()));

    let cli = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?;

    let res = cli
        .post("https://passport.bilibili.com/qrcode/getLoginInfo")
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

        let config = UserConfig::fetch(cookie_str).await?;
        println!("User Config: {:?}", config);

        save_user_config(&config)?;

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct RoomInitResponse {
    code: u8,
    msg: String,
    message: String,
    data: RoomInit,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct RoomResponse {
    code: u8,
    msg: String,
    message: String,
    data: Room,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct WsConfigResponse {
    code: u8,
    message: String,
    ttl: u8,
    data: WsConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]

struct BulletScreenPropertyResponse {
    code: u8,
    data: BulletScreenData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct BulletScreenData {
    property: BulletScreenConfig,
}

/// The first packet body sent to Bilibili Live
/// when websocket connection is established
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct FirstSecurityData {
    clientver: String,
    platform: String,
    protover: u64,
    uid: u64,
    roomid: i64,
    #[serde(rename = "type")]
    type_: u64,
}

/// Interface for Test Use
/// Connect to the given room id using the user credential we hold
/// and start the websocket client to monitor incoming messages
#[get("/roomInit/<roomId>")]
async fn roomInit(roomId: u64, state: &State<DanmujiState>) -> Result<String> {
    let state = state.config.lock().await;
    if let Some(state) = state.as_ref() {
        let ws = RoomConfig::fetch(roomId, state.raw_cookie.as_str())
            .await
            .unwrap();
        println!("{:?}", ws);

        // bullet screen config
        let bc = BulletScreenConfig::fetch(&ws, &state).await.unwrap();
        println!("{:?}", bc);

        // in a real application, we will communicate with DanmujiCore
        // and let the core module takes care of the following functionality
        // here we're experimenting the correct workflow
        std::thread::spawn(move || {
            let url = ws.get_ws_url().unwrap();
            println!("{}", url);
            let mut client = ClientBuilder::new(&url)
                .unwrap()
                .connect_insecure()
                .unwrap();

            println!("Connection Established");

            // send enter message
            let security_data = FirstSecurityData {
                clientver: "1.14.0".to_string(),
                platform: "web".to_string(),
                protover: 1,
                uid: 0,
                roomid: ws.room_init.room_id,
                type_: 2,
            };
            let mut data_bytes = serde_json::to_vec(&security_data).unwrap();
            let data_length = data_bytes.len();
            // header format:
            // offset    length    type    endian    name           note
            // 0         4         i32     Big       packet-length
            // 4         2         i16     Big       header-length  Fixed 16
            // 6         2         i16     Big       proto-version
            // 8         4         i32     Big       Opertion Type
            // 12        4         i32     Big       Seq ID         Fixed 1
            // reference: https://github.com/lovelyyoshino/Bilibili-Live-API/blob/master/API.WebSocket.md
            let mut header = vec![];
            // write packet length = header length + data length
            header
                .write_u32::<BigEndian>(16 + data_length as u32)
                .unwrap();
            // write header length
            header.write_u16::<BigEndian>(16).unwrap();
            // write proto-version 1
            header.write_u16::<BigEndian>(1).unwrap();
            // write operatin type: enter room = 7
            header.write_u32::<BigEndian>(7).unwrap();
            // write seq id
            header.write_u32::<BigEndian>(1).unwrap();
            println!("header: {:?}", header);

            header.append(&mut data_bytes);

            client.send_message(&Message::binary(header)).unwrap();

            for message in client.incoming_messages() {
                println!("Recv: {:?}", message.unwrap());
            }
        });
    }

    Ok("Hello".to_string())
}

struct DanmujiState {
    config: Arc<Mutex<Option<UserConfig>>>,
}

fn rocket(state: DanmujiState) -> rocket::Rocket<Build> {
    rocket::build()
        .attach(CORS)
        .manage(state)
        .mount("/", routes![index, getQrCode, loginCheck, logout, roomInit])
}

#[rocket::main]
async fn main() {
    let config = load_user_config();

    println!("{:?}", config);

    let state = DanmujiState {
        config: Arc::new(Mutex::new(config)),
    };

    let server = rocket(state);

    let _result = server.launch().await;

    // this is reachable only after `Shutdown::notify()` or `Ctrl+C`.
    println!("Rocket: deorbit.");
}

fn save_user_config(config: &UserConfig) -> Result<()> {
    let options = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("config.json")
        .unwrap();

    let writer = BufWriter::new(options);

    serde_json::to_writer(writer, config).unwrap();

    Ok(())
}

fn load_user_config() -> Option<UserConfig> {
    // try to restore config from file
    let file = OpenOptions::new().read(true).open("config.json");

    if let Ok(file) = file {
        let reader = BufReader::new(file);
        // if parse failed, return None
        // otherwise return config object
        serde_json::from_reader(reader).ok()
    } else {
        // no file found, return None
        None
    }
}
