use crate::{
    error::DanmujiError, BulletScreenPropertyResponse, DanmujiResult, RoomInitResponse,
    RoomResponse, UserInfoResponse, WsConfigResponse, USER_AGENT,
};
use std::collections::HashMap;

use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    // cookie used to access bilibili server
    pub raw_cookie: String,
    // user information
    pub user: User,
    pub cookie: Cookie,
}

impl UserConfig {
    /// fetch needed information to construct a UserConfig
    /// Object
    pub async fn fetch(raw_cookie: String) -> DanmujiResult<UserConfig> {
        // fetch user information
        let cli = reqwest::Client::new();
        let res = cli
            .get("https://api.live.bilibili.com/User/getUserInfo")
            .header("user_agent", USER_AGENT)
            .header("cookie", &raw_cookie)
            .send()
            .await?;

        let user_info: UserInfoResponse = res.json().await?;
        let user = user_info.data;
        let cookie = Cookie::from_str(&raw_cookie)?;
        Ok(UserConfig {
            raw_cookie,
            user,
            cookie,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/User.ts")]
pub struct User {
    pub uid: u64,
    pub uname: String,
    pub silver: u64,
    pub gold: u64,
    pub face: String,
    pub achieve: u64,
    pub vip: u64,
    pub svip: u64,
    pub user_level: u64,
    pub user_next_level: u64,
    pub user_intimacy: u64,
    pub user_next_intimacy: u64,
    pub user_level_rank: u64,
    pub user_charged: u64,
    pub billCoin: u64,
}

// todo: we can also parse expire date here
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    DedeUserID: String,
    bili_jct: String,
    DedeUserID__ckMd5: String,
    sid: String,
    SESSDATA: String,
}

impl Cookie {
    /// parse cookie from raw string
    pub fn from_str(raw_cookie: &str) -> DanmujiResult<Cookie> {
        let mut cookie_map = HashMap::new();

        let raw_token = raw_cookie.split(";");
        for token in raw_token {
            let kv_pair: Vec<&str> = token.split("=").collect();
            if kv_pair.len() == 2 {
                cookie_map.insert(kv_pair[0], kv_pair[1]);
            }
        }

        println!("{:?}", cookie_map);

        let cookie = Cookie {
            DedeUserID: cookie_map
                .get("DedeUserID")
                .ok_or(DanmujiError::cookie("Missing DedeUserID"))?
                .to_string(),
            bili_jct: cookie_map
                .get("bili_jct")
                .ok_or(DanmujiError::cookie("Missing bili_jct"))?
                .to_string(),
            DedeUserID__ckMd5: cookie_map
                .get("DedeUserID__ckMd5")
                .ok_or(DanmujiError::cookie("Missing DedeUserID__ckMd5"))?
                .to_string(),
            sid: cookie_map
                .get("sid")
                .ok_or(DanmujiError::cookie("Missing sid"))?
                .to_string(),
            SESSDATA: cookie_map
                .get("SESSDATA")
                .ok_or(DanmujiError::cookie("Missing SESSDATA"))?
                .to_string(),
        };

        Ok(cookie)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomConfig {
    pub room_init: RoomInit,
    pub room: Room,
}

impl RoomConfig {
    /// fetch needed information to construct RoomConfig
    pub async fn fetch(room_id: i64) -> DanmujiResult<RoomConfig> {
        // room init
        // get room_id and shrot_id
        // api reference: https://github.com/lovelyyoshino/Bilibili-Live-API/blob/master/API.room_init.md
        let mut dheaders = HeaderMap::new();
        dheaders.insert("user-agent", HeaderValue::from_str(USER_AGENT)?);

        let cli = reqwest::ClientBuilder::new()
            .default_headers(dheaders)
            .build()?;
        let res = cli
            .get(format!(
                "https://api.live.bilibili.com/room/v1/Room/room_init?id={}",
                room_id
            ))
            .send()
            .await?;
        let res: RoomInitResponse = res.json().await?;
        let room_init = res.data;

        // room data
        let effective_room_id = room_init.effective_room_id();
        let res = cli
            .get(format!(
                "https://api.live.bilibili.com/room_ex/v1/RoomNews/get?roomid={}",
                room_id
            ))
            .header(
                "referer",
                format!("https://live.bilibili.com/{}", effective_room_id),
            )
            .send()
            .await?;
        let res: RoomResponse = res.json().await?;
        let room = res.data;

        Ok(RoomConfig { room_init, room })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInit {
    pub room_id: i64,
    pub short_id: i64,
    pub uid: i64,
    pub need_p2p: i32,
    pub is_hidden: bool,
    pub is_locked: bool,
    pub is_portrait: bool,
    // 0 -> is live
    // 1 -> is not live
    // 2 -> streaming recorded vedio
    pub live_status: i32,
    pub hidden_till: i32,
    pub lock_till: i32,
    pub encrypted: bool,
    pub pwd_verified: bool,
    pub live_time: i64,
    pub room_shield: i32,
    pub is_sp: i32,
    pub special_type: i32,
}

impl RoomInit {
    pub fn effective_room_id(&self) -> i64 {
        if self.short_id > 0 {
            self.short_id
        } else {
            self.room_id
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/Room.ts")]
pub struct Room {
    // room id
    pub roomid: String,
    // streamer's user id
    pub uid: String,
    // live's title
    pub content: String,
    // unknown time field
    pub ctime: String,
    // unknown stats
    pub status: String,
    // streamer's user name
    pub uname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WsConfig {
    pub group: String,
    pub business_id: i32,
    pub refresh_row_factor: f64,
    pub refresh_rate: i32,
    pub max_delay: i64,
    pub token: String,
    pub host_list: Vec<WsHost>,
}

impl WsConfig {
    pub fn get_ws_url(&self) -> Option<String> {
        if self.host_list.is_empty() {
            return None;
        }

        let mut res = String::new();
        let index: usize = rand::thread_rng().gen_range(0..self.host_list.len());
        let host = self.host_list.get(index).unwrap();
        // build string
        // ws://<host>:<port>/sub
        res.push_str("ws://");
        res.push_str(&host.host);
        res.push_str(":");
        res.push_str(&format!("{}", host.ws_port));
        res.push_str("/sub");
        Some(res)
    }

    pub fn get_wss_url(&self) -> Option<String> {
        if self.host_list.is_empty() {
            return None;
        }

        let mut res = String::new();
        let index: usize = rand::thread_rng().gen_range(0..self.host_list.len());
        let host = self.host_list.get(index).unwrap();
        // build string
        // ws://<host>:<port>/sub
        res.push_str("wss://");
        res.push_str(&host.host);
        res.push_str(":");
        res.push_str(&format!("{}", host.wss_port));
        res.push_str("/sub");
        Some(res)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WsHost {
    pub host: String,
    pub port: i64,
    pub wss_port: i64,
    pub ws_port: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulletScreenConfig {
    pub bubble: i64,
    pub bubble_color: String,
    pub danmu: BulletScreen,
    pub uname_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulletScreen {
    pub color: i64,
    pub length: i64,
    pub mode: i64,
    pub room_id: i64,
}

impl BulletScreenConfig {
    pub async fn fetch(room: &RoomConfig, user: &UserConfig) -> DanmujiResult<BulletScreenConfig> {
        let cli = reqwest::Client::new();
        let res = cli
            .get(format!(
                "https://api.live.bilibili.com/xlive/web-room/v1/index/getInfoByUser?room_id={}",
                room.room_init.effective_room_id()
            ))
            .header("user-agent", USER_AGENT)
            .header(
                "referer",
                format!(
                    "https://live.bilibili.com/{}",
                    room.room_init.effective_room_id()
                ),
            )
            .header("cookie", user.raw_cookie.as_str())
            .send()
            .await?;

        let res: BulletScreenPropertyResponse = res.json().await?;
        Ok(res.data.property)
    }
}
