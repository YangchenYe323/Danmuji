//! Public Types of the client module, these types are the
//! public APIs through which users interact with Bilibili's Message
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use super::{BiliWebsocketInner, NotificationBody};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use ts_rs::TS;

/// The type representing a Bilibili's message received
/// by the client.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/BiliMessage.ts")]
#[serde(tag = "type", content = "body")]
pub enum BiliMessage {
    /// Someone sent a Bullet Screen Comment
    Danmu(DanmuMessage),
    /// Someone sent gifts
    Gift(GiftMessage),
    // Auto Room Popularity Update
    RoomPopularity(i32),
}

/// The type representing a bullet screen message
#[derive(Debug, Clone, Getters, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/DanmuMessage.ts")]
pub struct DanmuMessage {
    // sender's uid
    uid: u64,
    // sender's user name
    uname: String,
    // content of the bullet screen comment
    content: String,
    // is this message auto-generated for
    // gift sending?
    // (自动生成的xxx投喂了xxx的弹幕)
    is_gift_auto: bool,
    // timestamp the message is sent
    sent_time: u64,
    // is sender a manager(房管) of the room?
    is_manager: bool,
    // is sender a vip(老爷)?
    is_vip: bool,
    // is sender a svip(年费老爷)?
    is_svip: bool,
    // 正式会员
    is_full_member: bool,

    // 勋章，可能未佩戴
    #[getter(skip)]
    medal: Option<Medal>,

    // 用户等级 & 等级排名
    ul: u64,
    ul_rank: String,

    // 舰队身份
    guard: GuardType,
}

impl DanmuMessage {
    pub fn has_medal(&self) -> bool {
        self.medal.is_some()
    }

    pub fn medal_level(&self) -> Option<u64> {
        self.medal.as_ref().map(|m| m.level)
    }

    pub fn medal_name(&self) -> Option<&'_ str> {
        self.medal.as_ref().map(|m| m.name.as_str())
    }

    pub fn medal_streamer_name(&self) -> Option<&'_ str> {
        self.medal.as_ref().map(|m| m.streamer_name.as_str())
    }

    pub fn medal_streamer_roomid(&self) -> Option<u64> {
        self.medal.as_ref().map(|m| m.streamer_roomid)
    }
}

impl DanmuMessage {
    fn from_raw(value: &NotificationBody) -> Option<DanmuMessage> {
        let info = value.get("info")?;
        let info = info.as_array()?;
        let danmu_info = info.get(0)?.as_array()?;

        let is_gift_auto = danmu_info.get(9)?.as_u64().unwrap_or(0);
        let is_gift_auto = is_gift_auto == 2;
        let sent_time = danmu_info
            .get(4)
            .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
            .as_u64()
            .unwrap_or(0);

        // 用array传是哪个天才想出来的？
        let sender_info = info[2].as_array()?;
        // uid
        let uid = sender_info[0].as_u64().unwrap_or(0);
        // 用户名
        let uname = sender_info[1].as_str().unwrap_or("B站用户");
        let uname = uname.to_string();
        // 房管: 0 -> 非, 1 -> 是
        let is_manager = sender_info[2].as_u64().unwrap_or(0);
        let is_manager = is_manager == 1;
        // vip: 0 -> 非, 1 -> 是
        let is_vip = sender_info[3].as_u64().unwrap_or(0);
        let is_vip = is_vip == 1;
        // 年费vip: 0 -> 非, 1 -> 是
        let is_svip = sender_info[4].as_u64().unwrap_or(0);
        let is_svip = is_svip == 1;
        // 正式会员: 5000->非 10000->是
        let is_full_member = sender_info[5].as_u64().unwrap_or(5000);
        let is_full_member = is_full_member == 10000;

        // 弹幕内容
        let content = info[1].as_str().unwrap_or("");
        let content = content.to_string();

        // 勋章
        let medal_info = info[3].as_array();
        let medal = if let Some(medal) = medal_info {
            // 勋章可能是[]
            if medal.len() >= 4 {
                let level = medal[0].as_u64().unwrap_or(0);
                let name = medal[1].as_str().unwrap_or("").to_string();
                let streamer_name = medal[2].as_str().unwrap_or("").to_string();
                let streamer_roomid = medal[3].as_u64().unwrap_or(0);
                Some(Medal {
                    level,
                    name,
                    streamer_name,
                    streamer_roomid,
                })
            } else {
                None
            }
        } else {
            None
        };

        // 用户等级
        let ul_info = info[4].as_array();
        let (ul, ul_rank) = if let Some(ul_info) = ul_info {
            let ul = ul_info[0].as_u64().unwrap_or(0);
            let ul_rank = ul_info[1].as_str().unwrap_or("");
            let ul_rank = ul_rank.to_string();
            (ul, ul_rank)
        } else {
            (0, ">50000".to_string())
        };

        let guard_info = info[7].as_u64().unwrap_or(0);
        // println!("{}", guard_info);
        let guard: GuardType = guard_info.into();

        Some(DanmuMessage {
            uid,
            uname,
            content,
            is_gift_auto,
            sent_time,
            is_manager,
            is_vip,
            is_svip,
            is_full_member,
            medal,
            ul,
            ul_rank,
            guard,
        })
    }
}

impl DanmuMessage {
    pub fn default_message() -> Self {
        DanmuMessage {
            uid: 0,
            uname: "测试用户".to_string(),
            content: "你好Bilibili".to_string(),
            is_gift_auto: false,
            sent_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            is_manager: true,
            is_vip: true,
            is_svip: true,
            is_full_member: true,
            medal: Some(Medal {
                level: 40,
                name: "哈哈哈".to_string(),
                streamer_name: "".to_string(),
                streamer_roomid: 0,
            }),
            ul: 37,
            ul_rank: "".to_string(),
            guard: GuardType::Captain,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/Medal.ts")]
pub struct Medal {
    level: u64,
    name: String,
    streamer_name: String,
    streamer_roomid: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/GuardType.ts")]
pub enum GuardType {
    // 不是大航海
    NoGuard,
    // 舰长
    Captain,
    // 提督
    Admiral,
    // 总督
    Governor,
}

impl From<u64> for GuardType {
    fn from(num: u64) -> GuardType {
        match num {
            1 => GuardType::Governor,
            2 => GuardType::Admiral,
            3 => GuardType::Captain,
            _ => GuardType::NoGuard,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Getters, TS)]
#[ts(export)]
#[ts(export_to = "frontend/src/bindings/GiftMessage.ts")]
pub struct GiftMessage {
    uid: u64,
    uname: String,
    guard: GuardType,
    gift_id: u64,
    gift_name: String,
    // currently ts_rs exports Rust's u64 to Typescript's bigint,
    // which is not desirable because when parsing json data, the real
    // type is still Number.
    #[ts(type = "number")]
    gift_num: u64,
}

impl GiftMessage {
    fn from_raw(value: &NotificationBody) -> Option<GiftMessage> {
        assert_eq!("SEND_GIFT", value.get("cmd")?.as_str()?);

        let data = value.get("data")?;
        // user info
        let uid = data.get("uid")?.as_u64()?;
        let uname = data.get("uname")?.as_str()?.to_string();
        let guard: GuardType = data.get("guard_level")?.as_u64()?.into();

        // gift info
        let combo_send_info = data.get("combo_send")?;
        let gift_id = combo_send_info.get("gift_id")?.as_u64()?;
        let gift_name = combo_send_info.get("gift_name")?.as_str()?.to_string();
        let gift_num = combo_send_info.get("gift_num")?.as_u64()?;

        Some(GiftMessage {
            uid,
            uname,
            guard,
            gift_id,
            gift_name,
            gift_num,
        })
    }

    fn from_raw_combo(value: &NotificationBody) -> Option<GiftMessage> {
        assert_eq!("COMBO_SEND", value.get("cmd")?.as_str()?);

        let data = value.get("data")?;
        // user info
        let uid = data.get("uid")?.as_u64()?;
        let uname = data.get("uname")?.as_str()?.to_string();
        let guard: GuardType = data.get("medal_info")?.get("guard_level")?.as_u64()?.into();

        // gift info
        let gift_id = data.get("gift_id")?.as_u64()?;
        let gift_name = data.get("gift_name")?.as_str()?.to_string();
        let gift_num = data.get("combo_num")?.as_u64()?;

        Some(GiftMessage {
            uid,
            uname,
            guard,
            gift_id,
            gift_name,
            gift_num,
        })
    }
}

impl GiftMessage {
    pub fn default_message() -> GiftMessage {
        GiftMessage {
            uid: 0,
            uname: "测试用户".to_string(),
            guard: GuardType::Captain,
            gift_id: 0,
            gift_name: "小花花".to_string(),
            gift_num: 1,
        }
    }
}

impl BiliMessage {
    /// convert from websocket message body
    pub(crate) fn from_raw_wesocket_message(msg: BiliWebsocketInner) -> Option<BiliMessage> {
        let body = msg.into_body();
        match body {
            super::BiliWebsocketMessageBody::RoomPopularity(popularity) => {
                Some(BiliMessage::RoomPopularity(popularity))
            }
            super::BiliWebsocketMessageBody::Notification(notification) => {
                let cmd = notification.get("cmd")?;
                let cmd = cmd.as_str()?;
                // Current Commands:
                // Reference: https://github.com/lovelyyoshino/Bilibili-Live-API/blob/master/API.WebSocket.md
                // "DANMU_MSG": 弹幕
                // (欢迎消息触发不稳定，可能有缓存时间)
                // "ENTRY_EFFECT": 欢迎舰长
                // "WELCOME": 欢迎
                // "SUPER_CHAT_MESSAGE_JPN"
                // "SUPER_CHAT": SC
                //
                // "SEND_GIFT": 投喂礼物
                // "COMBO_SEND": 连击投喂 (不知道怎么触发)
                //
                // "GUARD_BUY": 上舰长
                // "USER_TOAST_MSG": 续费了舰长
                // "NOTICE_MSG": 本房间续费舰长
                //
                // 其他暂时不支持
                match cmd {
                    "DANMU_MSG" => {
                        DanmuMessage::from_raw(&notification).map(|msg| BiliMessage::Danmu(msg))
                    }

                    "SEND_GIFT" => {
                        GiftMessage::from_raw(&notification).map(|msg| BiliMessage::Gift(msg))
                    }

                    "COMBO_SEND" => {
                        GiftMessage::from_raw_combo(&notification).map(|msg| BiliMessage::Gift(msg))
                    }

                    "INTERACT_WORD" => None,

                    _ => {
                        // println!("{:?}", serde_json::to_string_pretty(&notification));
                        None
                    }
                }
            }
            super::BiliWebsocketMessageBody::EntryReply => None,
        }
    }
}
